# 자동 실행 검증 + 사용량 폴러 자동 복구 설계

날짜: 2026-06-16

## 배경

사용자가 두 가지를 요청했다. 조사 결과 두 요청은 서로 독립적인 작업이다.

1. **CodexBar exe가 Windows 시작 시 자동 실행되게 한다.**
2. **사용량이 가끔 갱신되지 않는 버그를 고친다.** 재현: 저녁까지 쓰고 PC를 켜둔 채 퇴근, 다음 날 아침 남은 사용량과 리셋 시각이 갱신돼 있지 않음. Claude를 계속 써도 동일. exe를 재시작하니 그제서야 갱신됨.

## 파트 A — Windows 시작 시 자동 실행

### 조사 결론: 코드 버그 아님. 검증만 필요.

자동 실행 기능은 이미 완전히 구현돼 있다.

- `rust/src/settings.rs`
  - `apply_start_at_login_registry(enabled)` — `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`에 `CodexBar` = `"<exe 경로>"`를 쓰거나 지운다.
  - `is_start_at_login_enabled()` — 레지스트리 값 존재 여부를 읽는다.
  - `Settings::load()`가 로드 시 `start_at_login`을 실제 레지스트리 상태와 동기화한다.
- 설정 화면 General 탭(`apps/desktop-tauri/src/surfaces/settings/tabs/GeneralTab.tsx`)에 "로그인 시 시작" 토글이 있고, `commands/settings.rs`를 거쳐 `set_start_at_login()` → `apply_start_at_login_registry()`로 연결된다.
- CLI `autostart --enable/--disable/--status` 서브커맨드도 존재한다.

사용자 PC 레지스트리 확인 결과 항목이 정상 등록돼 있었다:

```
CodexBar = "E:\04_Dev\02_tool\codexbar-zoey\target\release\codexbar-desktop-tauri.exe"
```

해당 exe도 존재한다(20.4MB, 2026-06-12 빌드). 즉 메커니즘상 다음 부팅 때 이미 자동 실행된다. 사용자는 토글을 켠 뒤 아직 재부팅 검증을 하지 않은 상태다.

### 작업

- **검증만 수행한다.** PC 재부팅 후 CodexBar가 트레이/플로팅바에 자동으로 뜨는지 확인.
- 재부팅 후에도 뜨지 않으면 그때 진짜 버그로 보고 원인을 조사한다(부팅 시 작업 디렉터리 문제, WebView2 초기화 타이밍 등). 현재 코드에서는 결함이 발견되지 않았다.

### 범위 외

- "신규 설치 시 기본 켬(default-on)"은 이번 작업에 포함하지 않는다. 현재 토글이 이미 켜져 있어 불필요하며, 사용자가 별도로 요청하면 그때 추가한다.

## 파트 B — 사용량 갱신 영구 정지 버그 수정

### 근본 원인

`apps/desktop-tauri/src-tauri/src/commands/providers.rs`의 프로바이더 폴링 루프가 인증 만료 응답을 받으면 **영구히 종료**된다.

- 프로바이더마다 `provider_poll_loop`가 돌며 주기적으로 사용량을 갱신한다.
- 폴 결과가 `UsagePollStatus::AuthExpired`로 분류되면 `ProviderPollerState::next_delay`가 `None`을 반환한다.
- 루프는 `None`을 받으면 `break`하여 해당 프로바이더의 폴링을 영구 종료한다. 이후 앱을 재시작하기 전까지 절대 다시 시도하지 않는다.

`UsagePollStatus`로 분류되는 인증 만료 경로:

- Claude OAuth(`rust/src/providers/claude/oauth.rs`): 토큰이 만료 5분 이내면 `is_expired()`가 true → **네트워크 호출 없이** 즉시 `ProviderError::AuthRequired` 반환. API가 401을 주는 경우에도 `AuthRequired`.
- `usage_bridge.rs::classify_provider_error`가 `AuthRequired` 등을 `UsagePollStatus::AuthExpired`로 매핑한다.

증상과의 일치: 밤사이 Claude 토큰이 만료 → 폴이 `AuthExpired` → 루프 `break` → 폴러 영구 정지 → 낮에 Claude를 계속 써서 토큰이 갱신돼도 폴러가 죽어 있어 모름 → exe 재시작 시 폴러가 새로 떠서 갱신 재개.

네트워크/rate-limit 오류는 백오프로 무한 재시도하므로 영구 정지하지 않는다. `AuthExpired`만 영구 정지한다.

### 자동 복구가 가능한 이유

`provider_poll_loop`는 매 회차 시작에서 `ProviderRefreshInputs::load()`로 설정·쿠키·API 키·토큰 계정을 다시 읽고, Claude `fetch_usage`도 매번 `load_credentials()`로 자격증명 파일을 다시 읽는다. 따라서 폴링을 느리게라도 계속 돌리면, 토큰이 갱신되는 순간(사용자가 Claude를 계속 쓰면 갱신됨) 다음 회차가 새 자격증명을 읽어 `Ok`가 되고 자동 복구된다. 또한 토큰이 로컬에서 만료된 상태에서는 `is_expired()`가 네트워크 호출 없이 즉시 반환하므로 재시도 비용이 사실상 없다.

### 수정 방식 (느린 재시도 + 백오프)

대상 파일: `apps/desktop-tauri/src-tauri/src/commands/providers.rs`

1. `ProviderPollerState`에 `auth_expired_backoff: BackoffPolicy` 필드를 추가한다. 기존 `bar/backoff.rs`를 재사용하며 정책은 base 60초 / 상한 15분 / factor 2.0 / jitter 0.2로 한다.
2. `next_delay`의 `AuthExpired` 분기를 `None`(영구 정지)에서 `Some(self.auth_expired_backoff.next_delay())`로 바꾼다.
3. `Ok` 분기에서 기존 `rate_limit_backoff`, `server_error_backoff`와 함께 `auth_expired_backoff`도 `reset()`한다.
4. `next_delay`가 더 이상 `None`을 반환하지 않으므로 반환 타입을 `Option<Duration>` → `Duration`으로 단순화한다. 폴링 루프의 죽은 `break` 분기(`let Some(delay) = ... else { ... break }`)와 "stopping automatic polling after auth-expired response" `tracing::warn` 로그를 제거한다. (이번 변경으로 죽게 된 코드만 정리한다.)

UI/트레이는 기존대로 `update_tray_and_notifications`가 `auth_expired` 상태를 반영하므로 별도 변경이 없다.

### 동작 흐름 (수정 후)

```
밤사이 토큰 만료 → 폴 결과 AuthExpired → break 안 함, 약 1분 후 재시도
→ 계속 만료면 2, 4, 8 … 분(상한 15분)으로 간격이 늘어남 (로컬 만료 시 네트워크 호출 없음)
→ 사용자가 Claude 사용해 토큰 갱신 → 다음 재시도가 새 토큰을 읽음 → Ok
→ 사용량/리셋 시각 갱신 재개 + 백오프 간격 리셋
```

### 검증 (성공 기준)

- 단위 테스트 추가: `AuthExpired`를 연속으로 받으면 `next_delay`가 점점 커지는 지연(상한 15분 이내)을 반환하고, 그 뒤 `Ok`를 받으면 base interval로 리셋됨을 확인한다.
- 기존 폴러 관련 테스트(`apps/desktop-tauri/src-tauri/src/commands/tests.rs`)가 깨지지 않는지 확인한다.
- `cargo test --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml`를 통과한다.
- 빌드 시 `pnpm run check-locale` 영향 없음(UI 문자열 변경 없음).

## 작업 순서

1. 파트 B 수정 (TDD: 실패하는 테스트 → 구현 → 통과).
2. 셸 크레이트 테스트 실행으로 검증.
3. 파트 A는 사용자가 재부팅으로 직접 검증.
