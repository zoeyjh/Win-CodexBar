# Floating Token Bar (codexbar-zoey) — 설계

**작성일**: 2026-05-28
**작성자**: Zoey Han
**상태**: Codex 리뷰 1회 반영 (v0.2)

## 1. 배경 및 목표

[Win-CodexBar](https://github.com/Finesssee/Win-CodexBar) (Tauri/React/Rust)을 사용 중인데 floating bar가 간헐적으로 사라지는 이슈가 있다. 원인은 미상이다. Fork해서 MVP 수준으로 단순화·안정화한 개인 도구를 만든다.

**핵심 목표**
1. Claude(Code) / Codex(CLI) / GitHub Copilot 3개 provider의 잔여 토큰을 화면 위 항상 보이는 minimal bar에 표시 (리셋 시간은 hover 툴팁에서)
2. 지원 시나리오 내에서 bar 자동 감지/복구 (watchdog) + 원인 진단을 위한 lifecycle 로깅
3. 큰 화면(Detail view)에서 리셋 주기 내 사용량 그래프 / 세션 로그 / 리셋 캘린더 확인

**Bar 표시 지원 범위 (= "사라지지 않게"가 적용되는 환경)**
- 일반 데스크톱 사용 (창 모드 앱 위, 멀티 모니터, 잠금 해제 후)
- DWM compose 정상 동작 환경
- Sleep/wake 복귀 후 자동 재배치

**비지원 시나리오 (정상적인 사라짐, watchdog가 강제 표시 시도하지 않음)**
- Windows exclusive fullscreen 게임/앱 (`MARGINS:-1` 류)
- UAC secure desktop (UAC 프롬프트 표시 중)
- RDP 세션 전환·로그오프 화면
- 가상 데스크톱 전환 (사용자가 다른 vdesk로 이동 시)
- 디스플레이 드라이버 재시작 중

비지원 시나리오에서는 watchdog가 "비지원 상황 감지" 이벤트만 로깅하고 bar 재생성을 시도하지 않는다.

**Non-goals (post-MVP에서 점진 확장)**
- 비용 추정, 모델별 분해, 알림 임계치 UI
- 트레이 아이콘 동적 미터, click-through, 테마 옵션, i18n
- Claude/Codex/Copilot 외 46개 provider
- E2E 자동화 테스트

## 2. 접근 방식

**선택: Selective Replace (β)**

Win-CodexBar의 데이터 레이어(provider 인증·폴링·DPAPI 저장)는 그대로 활용하고, floating bar UI와 Tauri 윈도우 생성 코드는 완전 재작성한다. Detail view는 MVP 3-카드 구성으로 단순화한다.

**근거**
- 사용자 핵심 통증 = 사라짐 = bar 윈도우 메커니즘 신뢰 불가 → 그 부분만 갈아엎는 게 합리적
- claude.ai 쿠키 추출 / OAuth / GitHub device flow 등 데이터 레이어는 처음부터 만들기 비용이 큼
- 49개 → 3개로 축소해 빌드/유지보수 부담 감소

**기각된 대안**
- α (Minimal Fork): 사라짐 원인이 Tauri 윈도우 코드 자체일 가능성에 대응 못 함
- γ (Data-Layer-Only Extract): 사실상 신규 제작. MVP 목적엔 과도

## 3. 프로젝트 경로 및 git 전략

- **코드 루트**: `E:\04_Dev\02_tool\codexbar-zoey\`
- **Spec/docs**: `E:\04_Dev\02_tool\codexbar-zoey\docs\`
- **저장소**: Win-CodexBar를 clone → upstream remote 별도 지정. fork는 안 함(sync 받을 계획 없음). cherry-pick은 필요 시 upstream에서 끌어옴
- **브랜치**: `master`(upstream snapshot 유지) + `zoey-mvp`(작업 브랜치)
- **라이선스**: MIT. 원본 Win-CodexBar의 `LICENSE` 파일과 copyright/notice를 그대로 보존하고, 신규 파일에는 자신의 copyright 추가

## 4. 선결 조사 (Pre-Implementation Research)

본격 구현 전에 다음 항목을 코드/실험으로 확인해야 한다. 결과에 따라 §5/§6의 일부 결정이 변경될 수 있다.

| R-# | 조사 항목 | 영향 | DoD |
|---|---|---|---|
| R-1 | **Provider별 실제 데이터 가능 필드**: Claude(claude.ai)/Codex CLI(local JSONL)/Copilot(GitHub API)이 각각 어떤 필드(used/limit/reset_at/sessions)를 제공하는지 | UsageSnapshot 스키마 확정, Card 1/2 가능 여부 | 각 provider별 가능/불가 필드 표 작성 |
| R-2 | **세션 로그 원천**: 각 CLI가 로컬에 세션 로그를 남기는지(예: `~/.codex/logs/*.jsonl`, `~/.claude/...`, Copilot CLI). 없으면 Card 2 대체안 필요 | DetailView Card 2 설계 | "있음/없음/대체" 결정 |
| R-3 | **Win-CodexBar 49 provider 의존성 그래프**: providers 디렉터리 외에 registry/enum/UI mapping/aggregation에 49개가 박혀 있는지 | §5.6 삭제 범위 | 영향 파일 목록 |
| R-4 | **Tauri window API 가능성**: `is_always_on_top()`, `is_visible()`, `current_monitor()`, work area 조회가 Tauri v1/v2 어디서 가능한지. 가려짐(occlusion) 감지는 가능한가 | §5.3 watchdog 헬스 기준 | 사용 가능 API 목록 |
| R-5 | **차트 라이브러리**: 기존 코드베이스에 이미 들어있는 차트 라이브러리 확인 후 그것 사용 | §5.5 Card 1 | 라이브러리 결정 |

R-1, R-2 결과가 부정적이면(예: Card 2 데이터 원천 없음) DetailView 카드 구성을 다시 협의한다.

## 5. 아키텍처 모듈 변경 맵

```
codexbar-zoey/
├── apps/desktop-tauri/
│   ├── src/
│   │   ├── floatbar/         # ⚡ 완전 재작성 (ultra-minimal bar)
│   │   ├── surfaces/         # ⚡ Detail view 3-카드 MVP로 단순화
│   │   ├── components/       # 일부 유지(공용 UI)
│   │   └── (assets/hooks/i18n/icons/lib/test/types — 유지)
│   └── src-tauri/
│       ├── tauri.conf.json   # ⚡ capabilities/security 만 정의 (window 정의 X)
│       └── src/              # ⚡ watchdog + lifecycle logger 추가
└── rust/src/
    ├── providers/            # ✅ claude/codex/copilot만 남기고 46개 디렉터리 제거 (R-3 후)
    ├── browser/              # ✅ 유지 (쿠키 추출)
    ├── status.rs             # ✅ 유지 (폴링) — backoff 보강
    └── tauri_app/            # ⚡ window lifecycle, state machine 보강
```
범례: ⚡ 재작성/수정, ✅ 유지

## 6. 컴포넌트 명세

### 6.1 FloatBar (React, 재작성)
- **역할**: 항상 위에 떠있는 ultra-minimal bar UI 렌더링
- **표시**: 3개 provider 아이콘(MVP는 prototyping용 도형 SVG) + 각 옆에 잔여 %. **리셋 시간은 bar에 표시하지 않음**(hover로만 노출)
- **인터랙션**:
  - 아이콘 hover (250ms) → 툴팁 (리셋까지 남은 시간 + 플랜/모델)
  - 빈 영역 mousedown → drag candidate. 같은 위치에서 mouseup이면 click, 5px 이상 이동했으면 `startDragging()`으로 전환 (drag threshold)
  - click(드래그 아닌 좌클릭) → Detail view 토글
  - 우클릭 → context menu (`설정`, `종료`)
- **입력**: Tauri 이벤트 `usage:update`의 provider별 `UsageSnapshot`
- **의존**: `@tauri-apps/api/window`, `@tauri-apps/api/event`

### 6.2 BarWindow (Rust, 재작성)
- **역할**: bar 윈도우 생성/관리. `tauri.conf.json`에는 window 정의를 두지 않고 capabilities/security만 둔다. **런타임에 `WebviewWindowBuilder`로 생성**해 모니터·DPI 변경 대응
- **속성**: `decorations:false, transparent:true, alwaysOnTop:true, skipTaskbar:true, resizable:false, focused:false`
- **상태 영구화**: `~/.codexbar-zoey/window.json` — `{ rect: {x, y, w, h}, monitor_id, dpi, saved_at, coord_space: "logical" }`
  - `monitor_id` 없거나 매칭 실패 → primary monitor로 fallback
  - 복원 시 현재 모니터의 work area로 clamp (음수 좌표, taskbar 영역 제외)
- **이벤트 발행**: Rust 내부 `tokio::sync::broadcast::Sender<LifecycleEvent>` (Tauri JS event가 아닌 typed enum). UI에 전달 필요한 일부만 `app.emit()`로 별도 발행

### 6.3 BarState (Rust, 신규)
명시적 상태 머신.

```rust
enum BarState {
    Visible,             // 정상 표시
    IntentionallyHidden, // 사용자가 트레이에서 `Bar 표시 토글`로 숨김
    Recovering,          // watchdog가 재생성 시도 중
    Paused,              // 5회 연속 재생성 실패 → 사용자 재시도 대기
    Quitting,            // 앱 종료 중
}
```

상태 전이는 단일 actor가 관리. Watchdog/Logger/Tray는 이 actor에 명령(`Show`, `Hide`, `Retry`, `Quit`)을 보내고 결과를 받는다.

### 6.4 BarWatchdog (Rust, 신규)
- **역할**: 1초 tick으로 헬스 체크. 비정상이면 재생성
- **상태 인식**: `BarState::Visible`일 때만 active. `IntentionallyHidden` / `Paused` / `Quitting`이면 no-op
- **헬스 기준** (R-4 결과 따라 조정):
  - 윈도우 핸들 유효
  - `is_visible()` = true
  - `is_always_on_top()` = true (API 미지원 시 마지막 setter 호출 후 시간으로 추정)
  - 현재 모니터들의 work area 안에 위치(off-screen 아님)
  - **알려진 한계**: 다른 fullscreen 창에 *가려진 상태*는 본 watchdog로 판별 불가. §1의 비지원 시나리오 참조
- **비지원 시나리오 감지**: exclusive fullscreen 앱 활성 / UAC secure desktop / RDP 전환 감지 시 `BarState::Visible`을 유지하되 재생성 시도 X, `lifecycle: skipped_recovery` 로깅
- **재생성 정책**: 헬스 실패 **3 tick 연속**(=3초) → `BarState::Recovering` → 마지막 정상 위치로 `BarWindow` 재생성. 성공 시 `Visible`로 복귀
- **재생성 5회 연속 실패**: `BarState::Paused` 전환 + 트레이 알림 + watchdog 일시 정지. 사용자가 트레이 `재시도` 메뉴 클릭 → `Visible`로 복귀 시도

### 6.5 LifecycleLogger (Rust, 신규)
- **역할**: 내부 `LifecycleEvent` 스트림 구독 → `~/.codexbar-zoey/logs/lifecycle-YYYY-MM-DD.log`에 JSON line append
- **이벤트 종류**: `created`, `shown`, `hidden`, `destroyed`, `always_on_top_changed`, `focus_changed`, `monitor_changed`, `state_transition`, `watchdog_tick`, `recovery_attempt`, `recovery_success`, `recovery_failed`, `skipped_recovery`, `error`
- **빈도 제어**: `watchdog_tick`은 5분에 1회 ok 1줄 + 모든 fail/recovery 1줄. 일반 이벤트는 발생 시 모두 기록
- **Redaction 규칙**:
  - 절대 기록 X: 쿠키, OAuth/API 토큰, refresh token, claude.ai session key, GitHub PAT, 이메일, 비밀번호
  - 기록 가능: provider 이름, 플랜 종류, 사용량 숫자, 에러 유형(HTTP status), 모니터 좌표, monitor_id
  - 디렉터리 경로는 사용자 홈을 `~`로 마스킹
- **파일 권한**: Windows ACL로 현재 사용자만 읽기/쓰기 가능 (DPAPI 폴더와 동일 정책)
- **로테이션**: 일자별 파일, 7일 후 자동 삭제
- **포맷 예**: `{"ts":"2026-05-28T08:42:13Z","event":"hidden","reason":"unknown","monitor":1,"pos":[1920,8]}`

### 6.6 DetailView (React, 재작성)
- **역할**: 큰 윈도우. 3개 카드 + 헤더에 provider 토글(다중 선택, 기본 전체 on)
  - **Card 1** "Plan remaining": 현재 리셋 주기 내 시계열 사용량 그래프. y축 = 잔여 %. 점선 = 현재 burn rate로 외삽한 projection
  - **Card 2** "Sessions": 세션별 로그 리스트 (시간/모델/토큰/디렉터리). **데이터 원천은 R-2 결과에 따라 결정**
  - **Card 3** "Next reset": provider별 다음 리셋 시각 + 주기 시각화
- **빈 상태 (각 카드 별도)**:
  - Card 1: "Waiting for first poll" (수동적 대기)
  - Card 2: "No sessions yet" (또는 R-2가 부정적이면 "Session log not available for this provider")
  - Card 3: "Sign in to see reset times" → Settings로 이동 버튼
- **윈도우**: 별도 `WebviewWindow`(`main`). bar click 또는 트레이 아이콘으로 토글
- **차트 라이브러리**: R-5 결과 사용

### 6.7 ProvidersService (Rust, 유지/축소)
- **유지**: `rust/src/providers/{claude,codex,copilot}`, `rust/src/browser/`, `rust/src/status.rs`
- **삭제 범위**: R-3 의존성 그래프 결과에 따라 결정. 단순 디렉터리 제거로 끝나지 않을 가능성 큼 (registry/enum/UI mapping/aggregator 손질 필요)
- **인터페이스**: `subscribe_usage() -> impl Stream<Item = UsageSnapshot>` — 기존 `status.rs` 위 wrapper
- **UsageSnapshot 스키마 (확장)**:

```rust
struct UsageSnapshot {
    provider: ProviderId,            // "claude" | "codex" | "copilot"
    plan: Option<String>,            // e.g. "Pro", "Plus", null if unknown
    unit: Unit,                      // "token" | "request" | "percent"
    used: Option<u64>,
    limit: Option<u64>,              // None = unbounded or unknown
    remaining_pct: Option<f32>,      // 0.0..=100.0
    reset_at: Option<DateTime<Utc>>, // None = unknown
    model_breakdown: Option<Vec<ModelUsage>>, // post-MVP
    status: SnapshotStatus,          // Ok / Stale / AuthExpired / RateLimited / Network / Unknown
    last_success_at: DateTime<Utc>,  // 마지막 성공 응답 시각 (Stale 판단용)
    confidence: Confidence,          // High (직접 응답) / Cached / Inferred
}
```

부분 데이터 정책: `Optional` 필드가 `None`이면 UI는 "—"로 표시하고 layout은 유지한다.

### 6.8 TrayIcon (Rust, 단순화)
- **역할**: 시스템 트레이 정적 로고 + 메뉴
- **메뉴**: `Detail 열기` / `Bar 표시 토글` (현재 상태 체크 표시) / `Watchdog 재시도` (Paused일 때만 활성) / `종료`
- **제거**: 기존 "이중 미터" 동적 아이콘 그리기

## 7. 데이터 흐름

**(A) 사용량 폴링 (백그라운드)**
```
타이머 (provider별 30~120s, jittered)
  → providers/{claude|codex|copilot}
  → UsageSnapshot (status 포함)
  → Rust state 저장 + history append + emit("usage:update")
  → FloatBar / DetailView 화면 갱신
```

기본 폴링 주기: claude.ai 60s, OpenAI 30s, Copilot 120s. ±10% jitter.

**Backoff 정책 (provider별 독립)**:
- 200 OK → 기본 주기
- 401/403 → 폴링 중지, 트레이 알림, 사용자 재인증까지 재시도 X
- 429 → `Retry-After` 우선. 없으면 exponential backoff (base 60s, max 30분, factor 2, ±20% jitter)
- 5xx / 네트워크 에러 → exponential backoff (base 폴링 주기, max 10분, factor 2)
- 성공 시 backoff 리셋

**(B) Bar 라이프사이클**
```
앱 시작 → BarState::Visible → BarWindow 생성 → LifecycleEvent::Created
       → LifecycleLogger (구독, redaction 후 로그 append)
       → BarWatchdog 1s tick (state == Visible일 때만)
       → 헬스 실패 3연속 → Recovering → 재생성 → Visible 복귀 (or 5회 실패 시 Paused)
```

**(C) 사용자 인터랙션**
- hover icon → React 툴팁 (250ms delay)
- mousedown on bar → drag candidate, 5px 이동 시 `startDragging()`
- click (no drag) → `invoke("toggle_detail")`
- 우클릭 → React context menu → `invoke("open_settings" | "quit")`
- 트레이 `Bar 표시 토글` → state IntentionallyHidden ↔ Visible

**(D) 영구 상태 (`~/.codexbar-zoey/`)**
- `window.json` — bar rect/monitor/dpi/coord_space
- `credentials.dpapi` — DPAPI 암호화 (기존 메커니즘 유지)
- `usage-cache.json` — 마지막 snapshot per provider (재시작 시 즉시 표시)
- `history/{provider}.jsonl` — Card 1 시계열 (timestamp, used, limit, remaining_pct). 리셋 시각 이후 entries만 keep + 14일 후 자동 trim
- `sessions/{provider}.jsonl` — Card 2 세션 로그 (R-2 결과로 enable 여부 결정. enable 시: timestamp, model, tokens, cwd_masked). 14일 후 trim
- `logs/lifecycle-YYYY-MM-DD.log` — 진단 로그 (7일 후 trim)

모든 파일은 현재 사용자 ACL로 제한.

## 8. 에러 처리

| 케이스 | 동작 |
|---|---|
| Provider 401/403 | 아이콘 dim + 툴팁 "재로그인 필요" + Detail 헤더 배너. 폴링 중지. |
| Provider 429 | `Retry-After` 또는 exponential backoff. 캐시값 표시 + 툴팁 "rate limited" |
| 5xx / 네트워크 끊김 | exponential backoff. 캐시값 표시 + 아이콘 ⚠️ 오버레이 |
| Bar 윈도우 이상 (지원 시나리오) | Watchdog 자동 재생성 |
| Bar 윈도우 이상 (비지원 시나리오 감지) | `skipped_recovery` 로깅, 재생성 X. 시나리오 종료 후 자동 복귀 |
| Bar 재생성 5회 연속 실패 | `BarState::Paused`, 트레이 알림, 트레이 `재시도` 메뉴로 재개 |
| Detail 윈도우 크래시 | Bar는 독립 동작, 재오픈 시 fresh state |
| `window.json` 손상 | 기본 위치(우상단)로 복구 |
| `usage-cache.json` 손상 | 무시하고 fresh fetch |
| `history/*.jsonl` 손상 | 해당 파일 백업(`.bak`) 후 새로 시작 |
| `credentials.dpapi` 손상 | 재인증 유도 |
| 첫 실행 / 자격 증명 없음 | Bar dim placeholder + Detail 자동 오픈 → 인증 가이드 |

**원칙**: 부분 실패는 부분 표시. 1개 provider 실패가 나머지를 막지 않는다.

**인증 UI 범위**: 첫 실행 인증 가이드는 기존 Win-CodexBar의 provider 연결 UI를 **유지하되**, 3개 provider만 보이도록 필터링. 완전 신규 인증 화면은 만들지 않는다.

## 9. 테스트 전략

### 단위 테스트 (Rust `#[cfg(test)]`)
- `BarWatchdog` 헬스 판정 로직: window trait를 mock으로 추상화해 정상/비정상/엣지(모니터 없음, 핸들 무효, work area 밖) 케이스 검증
- `BarState` 전이: 모든 (state, command) 페어가 의도된 결과 state로 가는지
- `ProvidersService` 응답 파싱: claude/codex/copilot mock 응답 → `UsageSnapshot` 변환, 부분 필드 케이스
- `LifecycleLogger` 로그 라인 JSON 스키마 검증 + redaction 패턴 (쿠키/토큰/이메일 등이 절대 안 새는지)
- Window position clamp: 음수 좌표, off-screen rect, monitor 사라짐 → 정상 좌표로 보정되는지
- Backoff 계산: 429/5xx 시퀀스 → 예상 대기 시간 + 성공 시 리셋

### 통합 테스트 (mockable)
- BarWindow trait + mock implementation으로 강제 hide → watchdog 3 tick(3초) 후 재생성 명령 확인
- 5회 재생성 실패 → `BarState::Paused` 전이 + 트레이 알림 호출 확인
- 비지원 시나리오 detector mock(`exclusive_fullscreen=true`) → 재생성 시도 안 함
- `window.json` 저장 → 로드 → 같은 rect 복원 (DPI 변경 시뮬레이션 포함)

### 수동 검증 체크리스트 (MVP DoD)
- [ ] 3 provider 잔여 % 정확히 표시
- [ ] hover 시 리셋 시간 정확
- [ ] click으로 Detail 열리고 3 카드(또는 R-2 결과 따라 조정된 구성) 모두 데이터 표시
- [ ] 5px 이상 drag로 옮긴 위치가 재시작 후 유지
- [ ] 멀티 모니터에서 의도한 모니터에 표시, 모니터 제거 시 primary로 복귀
- [ ] 일반 창 모드 앱 위에서 bar 유지 또는 watchdog 복구
- [ ] Sleep/wake 후 정상 동작
- [ ] 트레이 `Bar 표시 토글`로 숨김 → watchdog가 다시 띄우지 *않음*
- [ ] 1주일 사용 후 `lifecycle-*.log`로 사라짐 trigger 패턴 파악

### 진단 강제 시나리오 (사라짐 유발 시도)
- 다른 앱 fullscreen (창 모드/borderless/exclusive 구분), RDP 연결/끊기, 디스플레이 추가/제거, 절전, 가상 데스크톱 전환, UAC 프롬프트 → lifecycle 로그 패턴 관찰
- 지원/비지원 시나리오가 올바르게 분류되는지 확인

### Out of MVP
- E2E 자동화 (Tauri Driver 환경 구성 비용 큼) — 수동 검증으로 갈음
- Performance 벤치마크 (1초 tick CPU 영향: 측정만, target <0.5%)

## 10. Post-MVP 정리 단계

원인 특정 후:

| 항목 | 처리 |
|---|---|
| LifecycleLogger 로깅 수준 | 모든 lifecycle → 에러/recovery만 (디스크 절약 + 노이즈 감소) |
| BarWatchdog | 원인이 우리 코드면 **제거**. 외부 요인(OS/타사 앱)이면 **유지** (방어막) |
| 비지원 시나리오 탐지 | 운영 데이터로 false positive/negative 비율 확인 후 임계치 조정 |

## 11. 향후 점진 확장 (참고)

MVP 검증 후 우선순위 — 현 시점 결정 X, 사용 중 필요 따라:
- 비용 추정, 모델별 분해, 알림 임계치 UI
- 트레이 아이콘 동적 미터 복원
- 추가 provider (Gemini, Cursor 등)
- 테마 옵션, click-through, i18n
- 공식 logo 적용 (브랜드 가이드라인 확인 후)

## 12. 디자인 mock 및 오픈 이슈

### 디자인 mock 참조
- `docs/design/floatbar-mockup.html` — FloatBar 5개 상태 + hover 툴팁 시각
- `docs/design/detail-mockup.html` — DetailView 3 카드 + provider 토글 + empty states
- `docs/design/design-notes.md` — 색상 팔레트(HEX), 타이포(Pretendard/JetBrains Mono), 치수, 디자인 결정 근거
- 구현 시 색상/치수/폰트는 `design-notes.md`의 토큰 값을 그대로 사용

### MVP 결정 사항
- **Provider 로고**: MVP는 `design-notes.md` §7의 **prototyping용 도형 SVG 사용**. 공개 배포 시 공식 로고로 교체 검토 (post-MVP)
- **폰트 의존성**: **앱 번들에 Pretendard/JetBrains Mono 포함**. Google Fonts URL 의존 X (오프라인·CSP·로딩 지연 회피)
- **라이선스**: MIT. Win-CodexBar 원본 LICENSE/copyright 보존 + 신규 파일은 자체 copyright 헤더

### 남은 오픈 이슈 (구현 중 해결)
- R-1~R-5 선결 조사 결과
- **claude.ai 인증 안정성**: 쿠키 추출 방식은 브라우저 업데이트나 claude.ai 변경에 취약. 인증 끊김 빈도 모니터링 필요 (post-MVP 운영 데이터로 판단)
