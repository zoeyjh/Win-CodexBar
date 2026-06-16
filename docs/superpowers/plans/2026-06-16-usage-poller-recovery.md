# 사용량 폴러 자동 복구 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 인증 만료(AuthExpired) 응답을 받으면 사용량 폴러가 영구 정지하던 버그를, 백오프 기반 느린 재시도로 바꿔 자동 복구되게 한다.

**Architecture:** 기존 `bar/backoff.rs`의 `BackoffPolicy`를 재사용해 인증 만료 전용 백오프(base 60초, 상한 15분)를 추가한다. `ProviderPollerState::next_delay`가 `AuthExpired`에서 `None`(영구 정지) 대신 백오프 지연을 반환하도록 바꾸고, 반환 타입을 `Duration`으로 단순화한 뒤 폴링 루프의 죽은 `break` 분기를 제거한다.

**Tech Stack:** Rust, Tauri, tokio. 대상 크레이트: `codexbar-desktop-tauri` (`apps/desktop-tauri/src-tauri`).

스펙: `docs/superpowers/specs/2026-06-16-autostart-and-usage-poller-recovery-design.md`

---

## File Structure

- Modify: `apps/desktop-tauri/src-tauri/src/bar/backoff.rs` — 인증 만료 전용 백오프 정책 생성 함수 `auth_expired_backoff()` 추가.
- Modify: `apps/desktop-tauri/src-tauri/src/commands/providers.rs`
  - `ProviderPollerState`에 `auth_expired_backoff` 필드 추가, `next_delay` 반환 타입을 `Duration`으로 변경.
  - `provider_poll_loop`의 죽은 `break` 분기 + "stopping automatic polling" 로그 제거.
  - 파일 끝에 `#[cfg(test)] mod poller_tests` 추가(비공개 `ProviderPollerState`를 모듈 내부에서 직접 테스트).

파트 A(Windows 자동 실행)는 코드 변경이 없다. 마지막 Task에서 수동 검증 절차만 안내한다.

---

### Task 1: 인증 만료 백오프 정책 추가

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/bar/backoff.rs:49-51` (기존 `server_error_backoff` 아래에 추가)

- [ ] **Step 1: `auth_expired_backoff()` 함수 추가**

`server_error_backoff` 함수 바로 아래(파일 끝, 49-51줄 다음)에 추가한다:

```rust
pub fn auth_expired_backoff() -> BackoffPolicy {
    BackoffPolicy::new(
        Duration::from_secs(60),
        Duration::from_secs(15 * 60),
        2.0,
        0.2,
    )
}
```

- [ ] **Step 2: 컴파일 확인**

Run: `cargo build --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml`
Expected: 빌드 성공 (새 함수는 아직 미사용이라 `dead_code` 경고가 날 수 있음 — 다음 Task에서 사용하므로 무시).

커밋은 Task 3에서 폴러 변경과 함께 한 번에 한다(중간 미사용 경고 상태로 커밋하지 않기 위함).

---

### Task 2: 폴러에 인증 만료 백오프 연결 (TDD)

**Files:**
- Test: `apps/desktop-tauri/src-tauri/src/commands/providers.rs` (파일 끝, 552줄 다음에 새 모듈 추가)
- Modify: `apps/desktop-tauri/src-tauri/src/commands/providers.rs:132-167` (`ProviderPollerState` 구조체/`new`/`next_delay`)

- [ ] **Step 1: 실패하는 테스트 작성**

`providers.rs` 파일 맨 끝(552줄 `}` 다음)에 아래 모듈을 추가한다. `Duration`은 파일 상단(1줄)에서 이미 import되어 있다.

```rust
#[cfg(test)]
mod poller_tests {
    use super::*;
    use crate::usage_bridge::UsagePollStatus;

    // 인증 만료가 반복돼도 폴러는 멈추지 않고(반환 타입이 Duration), 지연은
    // 상한(15분) + 지터(+20%) 한도 안에서 점점 커져 상한 부근까지 도달한다.
    #[test]
    fn auth_expired_keeps_polling_with_capped_backoff() {
        let mut poller = ProviderPollerState::new(Duration::from_secs(60));
        let mut last = Duration::ZERO;
        for _ in 0..8 {
            last = poller.next_delay(UsagePollStatus::AuthExpired);
            assert!(last > Duration::ZERO);
            // 상한 900초 * (1 + 0.2 지터) = 1080초를 넘지 않는다.
            assert!(last <= Duration::from_secs(1080), "delay {last:?} exceeds cap+jitter");
        }
        // 충분히 반복하면 상한(900초) - 20% 지터 = 720초 이상에 도달한다.
        assert!(last >= Duration::from_secs(720), "delay {last:?} did not approach cap");
    }

    // Ok 응답을 받으면 인증 만료 백오프가 리셋되어 다음 인증 만료 지연이
    // 다시 첫 시도 범위(60초 ± 20%)로 돌아온다.
    #[test]
    fn ok_resets_auth_expired_backoff() {
        let mut poller = ProviderPollerState::new(Duration::from_secs(60));
        for _ in 0..6 {
            poller.next_delay(UsagePollStatus::AuthExpired);
        }

        let after_ok = poller.next_delay(UsagePollStatus::Ok);
        assert_eq!(after_ok, Duration::from_secs(60));

        let after_reset = poller.next_delay(UsagePollStatus::AuthExpired);
        assert!(
            after_reset >= Duration::from_secs(48) && after_reset <= Duration::from_secs(72),
            "delay {after_reset:?} not in first-attempt range"
        );
    }
}
```

- [ ] **Step 2: 테스트가 실패(컴파일 에러)하는지 확인**

Run: `cargo test --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml poller_tests`
Expected: 컴파일 실패. 현재 `next_delay`는 `Option<Duration>`을 반환하고 `auth_expired_backoff` 필드가 없어, `next_delay(...) > Duration::ZERO` 비교와 `assert_eq!(after_ok, Duration::from_secs(60))`에서 타입 불일치 에러가 난다.

- [ ] **Step 3: `ProviderPollerState` 구조체에 필드 추가**

`apps/desktop-tauri/src-tauri/src/commands/providers.rs:132-136`의 구조체를 아래로 교체한다:

```rust
struct ProviderPollerState {
    rate_limit_backoff: crate::bar::backoff::BackoffPolicy,
    server_error_backoff: crate::bar::backoff::BackoffPolicy,
    auth_expired_backoff: crate::bar::backoff::BackoffPolicy,
    base_interval: Duration,
}
```

- [ ] **Step 4: `new`에서 필드 초기화**

`providers.rs:139-145`의 `new`를 아래로 교체한다:

```rust
    fn new(base_interval: Duration) -> Self {
        Self {
            rate_limit_backoff: crate::bar::backoff::rate_limit_backoff(),
            server_error_backoff: crate::bar::backoff::server_error_backoff(base_interval),
            auth_expired_backoff: crate::bar::backoff::auth_expired_backoff(),
            base_interval,
        }
    }
```

- [ ] **Step 5: `next_delay` 반환 타입을 `Duration`으로 바꾸고 AuthExpired/Ok 분기 수정**

`providers.rs:154-166`의 `next_delay`를 아래로 교체한다:

```rust
    fn next_delay(&mut self, status: UsagePollStatus) -> Duration {
        match status {
            UsagePollStatus::Ok => {
                self.rate_limit_backoff.reset();
                self.server_error_backoff.reset();
                self.auth_expired_backoff.reset();
                self.base_interval
            }
            UsagePollStatus::RateLimited => self.rate_limit_backoff.next_delay(),
            UsagePollStatus::Network => self.server_error_backoff.next_delay(),
            UsagePollStatus::Unknown => self.base_interval,
            UsagePollStatus::AuthExpired => self.auth_expired_backoff.next_delay(),
        }
    }
```

- [ ] **Step 6: 테스트 통과 확인**

Run: `cargo test --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml poller_tests`
Expected: 컴파일은 통과하나, `provider_poll_loop`(190-226줄)가 아직 `let Some(delay) = poller.next_delay(status) else {...}`로 `Option`을 기대해 **그 함수에서 컴파일 에러**가 난다. 이는 Task 3에서 고친다. (poller_tests 자체 로직은 올바름.)

> 참고: Step 5까지 하면 루프 코드가 깨지므로 Task 2와 Task 3은 한 흐름으로 이어서 진행한다. 커밋은 Task 3 끝에서 한 번에 한다.

---

### Task 3: 폴링 루프의 죽은 break 제거 + 전체 검증/커밋

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/commands/providers.rs:217-224` (`provider_poll_loop`의 delay 처리부)

- [ ] **Step 1: 죽은 `break` 분기와 경고 로그 제거**

`providers.rs:217-224`의 아래 블록:

```rust
        let Some(delay) = poller.next_delay(status) else {
            tracing::warn!(
                provider = id.cli_name(),
                "stopping automatic polling after auth-expired response"
            );
            break;
        };
        tokio::time::sleep(delay).await;
```

을 다음으로 교체한다:

```rust
        let delay = poller.next_delay(status);
        tokio::time::sleep(delay).await;
```

- [ ] **Step 2: 셸 크레이트 전체 테스트 실행**

Run: `cargo test --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml`
Expected: PASS. 신규 `poller_tests` 2개 통과, 기존 `commands::tests`(`fetch_context_*`, `usage_snapshot_from_status_*`, `classify_provider_error_*` 등) 모두 통과.

- [ ] **Step 3: 포맷 + 린트**

Run: `cargo fmt --all`
Run: `cargo clippy --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml --all-targets -- -D warnings`
Expected: 경고 0. (Task 1에서 났을 수 있는 `auth_expired_backoff` 미사용 경고가 이제 사라진다.)

- [ ] **Step 4: 커밋**

> 사용자 CLAUDE.md 규칙: 사용자가 "커밋"을 명시 승인하기 전에는 `git commit` 금지. 이 단계는 **사용자 승인 후에만** 실행한다. 커밋 메시지는 사용자 검토를 받는다.

제안 커밋 메시지:

```bash
git add apps/desktop-tauri/src-tauri/src/bar/backoff.rs apps/desktop-tauri/src-tauri/src/commands/providers.rs
git commit -m "fix(poller): recover from auth-expired instead of stopping forever"
```

---

### Task 4: 파트 A — Windows 자동 실행 수동 검증 (코드 변경 없음)

**Files:** 없음.

- [ ] **Step 1: 레지스트리 등록 확인**

Run (PowerShell): `(Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run').CodexBar`
Expected: CodexBar exe 경로가 출력됨 (예: `"E:\04_Dev\02_tool\codexbar-zoey\target\release\codexbar-desktop-tauri.exe"`). 이미 등록돼 있음이 확인된 상태.

- [ ] **Step 2: 재부팅 검증**

사용자가 PC를 재부팅한 뒤 CodexBar가 트레이/플로팅바에 자동으로 뜨는지 확인한다.
- 정상 실행되면 파트 A는 완료(코드 작업 불필요).
- 재부팅 후에도 뜨지 않으면 그때 진짜 버그로 보고 별도 디버깅(부팅 시 작업 디렉터리, WebView2 초기화 타이밍 등)을 진행한다.

---

## Self-Review

- **Spec coverage:**
  - 파트 B 근본 원인(AuthExpired → `break` 영구 정지) → Task 2 Step 5 + Task 3 Step 1에서 제거/대체. ✓
  - 백오프(base 60초/상한 15분/factor 2.0/jitter 0.2) → Task 1. ✓
  - Ok 시 auth 백오프 리셋 → Task 2 Step 5 + 테스트. ✓
  - 반환 타입 `Option<Duration>`→`Duration` 단순화 + 죽은 break/로그 제거 → Task 2 Step 5, Task 3 Step 1. ✓
  - 단위 테스트(증가 후 리셋) → Task 2 Step 1. ✓
  - 셸 크레이트 테스트 통과 → Task 3 Step 2. ✓
  - 파트 A 검증 → Task 4. ✓
- **Placeholder scan:** TBD/TODO/"적절히 처리" 없음. 모든 코드 단계에 실제 코드 포함. ✓
- **Type consistency:** `auth_expired_backoff()` (backoff.rs) ↔ 필드 `auth_expired_backoff` (providers.rs) ↔ `next_delay` 반환 `Duration` — Task 1/2/3 전반에서 일관. 테스트는 `UsagePollStatus`를 `crate::usage_bridge`에서 import. ✓
