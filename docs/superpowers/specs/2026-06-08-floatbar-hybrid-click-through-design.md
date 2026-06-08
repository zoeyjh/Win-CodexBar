# FloatBar 하이브리드 클릭 통과 (Hybrid Click-Through)

- 날짜: 2026-06-08
- 브랜치: zoey-mvp
- 대상: 데스크톱 FloatBar 창 (`apps/desktop-tauri`)

## 문제 (Problem)

FloatBar 창은 380×220px(가로) / 80×280px(세로)의 **투명 창**이다. 실제 보이는 바
(`.floatbar` 엘리먼트)는 창 가운데에 작게 배치되고, 나머지 영역(특히 위쪽 툴팁
공간)은 투명한 빈 영역이다.

현재 상호작용 모드(`float_bar_click_through = false`)에서는 **창 전체**가 마우스
클릭을 받는다. 그래서:

- 바 위를 클릭 → 바가 동작 (detail 토글 등) — 정상
- 빈 투명 영역을 클릭 → 클릭이 창에 먹혀서 사라짐. 뒤에 있는 화면(바탕화면, 다른
  앱)으로 **전달되지 않음** — 버그

사용자는 "바 위 클릭은 바가 받고, 빈 영역 클릭은 뒤 화면으로 통과"되기를 원한다.

## 목표 (Goals)

1. 바 위 클릭은 FloatBar가 받고, 그 외 빈 영역 클릭은 뒤 창으로 통과시킨다(하이브리드).
2. 기존 "창 전체 통과(완전 통과)" 모드는 옵션으로 유지한다.
3. 사용자가 이 옵션을 켜고 끌 수 있는 경로를 MVP에서 제공한다.

## 비목표 (Non-Goals)

- macOS/Linux 동작 보장(이번 작업은 Windows 우선; `set_ignore_cursor_events`는
  크로스플랫폼이지만 검증은 Windows에서 한다).
- 툴팁/드래그/레이아웃 동작 변경.
- 새 설정 항목 추가나 설정 마이그레이션.

## 모드 정의 (Modes)

기존 `float_bar_click_through: bool` 설정을 그대로 사용하되 의미를 재정의한다.

| 설정값 | 모드 | 동작 |
|---|---|---|
| `false` (기본값) | **하이브리드** | 커서가 바 위 → 클릭됨 / 빈 영역 → 통과 |
| `true` | **완전 통과** | 창 전체가 항상 통과 (기존 동작) |

예전의 "창 전체가 클릭을 먹는" 모드(`false`의 기존 의미)는 사라진다. 아무도 원치
않는 동작이라 별도 보존하지 않는다. 설정 스키마 변경·마이그레이션 없음.

## 메커니즘: 커서 위치 폴링 (접근 A)

투명 WebView2 창에서 "내용 위만 클릭, 나머지 통과"를 구현하는 검증된 방식.
대안(WM_NCHITTEST 서브클래싱)은 WebView2 자식 HWND가 마우스를 직접 가로채서
신뢰도가 낮고, 창 리사이즈 방식은 깜빡임/복잡도가 크다.

Rust에서 80ms 주기로 폴링하는 루프:

1. `app.cursor_position()`으로 커서의 **물리 좌표**를 읽는다.
2. 프론트가 보고한 **바의 창-상대 논리 사각형** + 현재 창
   `outer_position()` + `scale_factor()`로 **바의 화면 물리 사각형**을 계산한다.
3. 커서가 사각형 안 → `window.set_ignore_cursor_events(false)` (클릭됨)
   / 밖 → `set_ignore_cursor_events(true)` (통과).
4. 직전 상태와 다를 때만 호출(불필요한 토글 방지).

창을 드래그로 옮겨도 매 틱마다 창 위치를 새로 읽으므로 화면 사각형이 따라온다.

## 컴포넌트 (Components)

### 1. 프론트엔드 바 영역 리포터
- 위치: `apps/desktop-tauri/src/floatbar/FloatBar.tsx`, `.../floatbar/api.ts`
- `.floatbar` 엘리먼트의 `getBoundingClientRect()`(CSS px = 창-상대 논리 좌표)를
  측정한다.
- 보고 시점: mount 시 1회 + `ResizeObserver`로 바 크기 변동 시(provider 수
  변화 등).
- `api.ts`에 `setFloatBarHitRect(rect)` 추가 → `invoke("set_float_bar_hit_rect", ...)`.

### 2. Rust command `set_float_bar_hit_rect`
- 위치: `apps/desktop-tauri/src-tauri/src/floatbar/commands.rs`
- 인자: `x, y, w, h` (창-상대 논리 px).
- 받은 사각형을 공유 상태에 저장한다.
- `main.rs`의 `invoke_handler`에 등록.

### 3. 공유 상태 (Hit rect)
- `Arc<Mutex<Option<HitRect>>>` 형태의 managed state.
- `HitRect { x, y, w, h }` — 창-상대 논리 좌표.
- 폴링 루프가 읽고, command가 쓴다.

### 4. 폴링 루프
- 위치: `apps/desktop-tauri/src-tauri/src/floatbar/window.rs`(또는 새 모듈
  `floatbar/hit_test.rs`).
- 하이브리드 모드일 때만 가동. 80ms 주기.
- `AtomicBool` 가동 플래그로 중복 spawn 방지.
- 창이 숨겨지거나(`get_webview_window`가 None) 모드가 완전 통과로 바뀌면 루프 종료.

### 5. 모드 배선
- `window::show()`: 하이브리드면 폴링 시작, 완전 통과면 `set_ignore_cursor_events(true)`
  고정 + 폴링 미가동.
- `commands::set_float_bar_click_through()`: 토글 변경 시 폴링 시작/정지 및
  ignore 상태를 즉시 반영.

### 6. tray "설정 열기" 진입로
MVP에서는 로그인 상태에서 Settings 창에 도달할 경로가 없다(tray에 설정 항목
없음, Detail 화면의 "Open Settings" 버튼은 provider 데이터가 0건일 때만 표시됨).
완전 통과 옵션을 사용자가 켜고 끄려면 진입로가 필요하다.

- `tray_menu.rs`: `TrayMenuEntry::item("open_settings", "설정 열기")` 추가.
- `tray_bridge.rs`: `MenuAction::OpenSettings` 변형 추가, `resolve_menu_action`에
  `"open_settings" => OpenSettings` 매핑, `handle_menu_event`에서
  `shell::settings_window::open_or_focus(app, "display")` 호출 → Settings 창의
  **Display 탭**(Float Bar 섹션 = Click-Through 토글이 있는 곳)을 연다.

### 7. 라벨 갱신
- `apps/desktop-tauri/src/floatbar/SettingsSection.tsx`: "Click-Through" 토글의
  라벨/설명을 새 의미에 맞게 수정한다.
  - 예) 라벨 "Full Click-Through", 설명 "켜면 바 전체를 통과시킵니다. 끄면 바 위만
    클릭되고 주변 빈 영역은 뒤 화면으로 통과합니다."

## 정리 작업 (Refactor, 목표 직결분만)

- 클릭 통과 토글을 raw Win32 `apply_click_through`(`WS_EX_TRANSPARENT` 직접 조작)
  대신 Tauri의 `window.set_ignore_cursor_events(bool)`로 통일한다. 같은
  `WS_EX_TRANSPARENT` 비트를 두 경로(폴링 + 완전 통과)가 만지면서 충돌하는 것을
  막기 위함이다.
- opacity용 raw Win32(`apply_opacity`, `WS_EX_LAYERED` + `SetLayeredWindowAttributes`)는
  그대로 둔다.

## 테스트 가능한 핵심 (순수 함수)

이 두 함수에 단위 테스트를 작성한다. 폴링 루프와 Tauri 호출은 얇은 껍데기로 분리.

- `window_rect_to_screen(hit_rect_logical, win_pos_physical, scale) -> ScreenRect`
  — 창-상대 논리 사각형을 화면 물리 사각형으로 변환.
- `point_in_rect(cursor_physical, screen_rect) -> bool` — 커서가 사각형 안인지 판정.

테스트 케이스 예:
- scale=1.0, 창 (100,100), hit rect (10,10,200,40) → 화면 사각형 (110,110,200,40).
- scale=2.0일 때 좌표·크기가 2배로 변환되는지.
- 커서가 경계 안/밖/모서리일 때 `point_in_rect` 판정.

## 엣지 케이스 (Edge Cases)

- **드래그 중**: 드래그 시작 시 커서가 바 위에 있어 interactive가 유지되고, 창이
  커서를 따라 움직여 사각형도 함께 이동하므로 별도 처리하지 않는다. (문제가
  드러나면 후속 처리.)
- **폴링 중복 시작 방지**: `AtomicBool` 가동 플래그로 `show` 재호출 시 중복 spawn
  안 함.
- **창 숨김 / 완전 통과 전환**: 루프가 조건을 감지하면 스스로 종료한다.
- **hit rect 미보고 상태**: 프론트가 아직 보고하지 않았으면(`None`) 폴링은 안전하게
  통과(ignore=true) 상태를 유지한다.

## 영향받는 파일 (Touch List)

- `apps/desktop-tauri/src/floatbar/FloatBar.tsx` — 바 영역 측정·보고
- `apps/desktop-tauri/src/floatbar/api.ts` — `setFloatBarHitRect`
- `apps/desktop-tauri/src/floatbar/SettingsSection.tsx` — 라벨/설명
- `apps/desktop-tauri/src-tauri/src/floatbar/commands.rs` — `set_float_bar_hit_rect`
- `apps/desktop-tauri/src-tauri/src/floatbar/window.rs` — 폴링 루프, 모드 배선,
  `set_ignore_cursor_events` 전환
- `apps/desktop-tauri/src-tauri/src/floatbar/mod.rs` — managed state 등록(필요 시)
- `apps/desktop-tauri/src-tauri/src/main.rs` — command 등록
- `apps/desktop-tauri/src-tauri/src/tray_menu.rs` — "설정 열기" 항목
- `apps/desktop-tauri/src-tauri/src/tray_bridge.rs` — OpenSettings 액션

## 성공 기준 (Success Criteria)

1. 하이브리드 모드(기본)에서 바 위 클릭 → 바 동작, 빈 영역 클릭 → 뒤 화면 동작.
2. 완전 통과 모드(`click_through=true`)에서 바 포함 창 전체가 통과.
3. tray "설정 열기" → Settings 창 Display 탭이 열리고 Click-Through 토글로 모드 전환 가능.
4. 바를 드래그로 옮긴 뒤에도 1번이 그대로 성립.
5. `window_rect_to_screen` / `point_in_rect` 단위 테스트 통과.
