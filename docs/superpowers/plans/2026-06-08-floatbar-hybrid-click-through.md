# FloatBar 하이브리드 클릭 통과 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** FloatBar 투명 창에서 바 위 클릭은 바가 받고, 빈 영역 클릭은 뒤 화면으로 통과시키며(하이브리드), 완전 통과 모드를 옵션으로 유지하고 tray에서 설정 진입로를 제공한다.

**Architecture:** Rust에서 80ms 주기로 마우스 커서가 바의 화면 사각형 안/밖인지 판정해 `set_ignore_cursor_events`를 토글한다. 바의 사각형은 프론트엔드(`FloatBar.tsx`)가 측정해 command로 보고한다. 좌표 변환·판정은 순수 함수로 분리해 단위 테스트한다.

**Tech Stack:** Rust(Tauri v2.10), Win32 `GetCursorPos`, React/TypeScript, Vitest.

**Spec:** `docs/superpowers/specs/2026-06-08-floatbar-hybrid-click-through-design.md`

---

## File Structure

신규/수정 파일과 책임:

- **신규** `apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs` — 순수 함수(좌표 변환·판정), 공유 상태(`FloatBarHitState`), 폴링 루프(start/stop), 모드 적용 헬퍼(`apply_click_through_mode`), Win32 커서 읽기.
- **수정** `apps/desktop-tauri/src-tauri/src/floatbar/mod.rs` — `hit_test` 모듈 등록·재노출, `apply_state`의 클릭통과 호출을 새 헬퍼로 교체.
- **수정** `apps/desktop-tauri/src-tauri/src/floatbar/commands.rs` — `set_float_bar_hit_rect` command 추가, `set_float_bar_click_through`를 새 헬퍼로 교체.
- **수정** `apps/desktop-tauri/src-tauri/src/floatbar/window.rs` — `show()`의 클릭통과 호출을 새 헬퍼로 교체, 미사용 `apply_click_through` 제거.
- **수정** `apps/desktop-tauri/src-tauri/src/main.rs` — `FloatBarHitState` manage 등록, `set_float_bar_hit_rect` handler 등록.
- **수정** `apps/desktop-tauri/src-tauri/src/tray_menu.rs` — "설정 열기" 항목.
- **수정** `apps/desktop-tauri/src-tauri/src/tray_bridge.rs` — `OpenSettings` 액션.
- **수정** `apps/desktop-tauri/src/floatbar/api.ts` — `setFloatBarHitRect`.
- **수정** `apps/desktop-tauri/src/floatbar/FloatBar.tsx` — 바 영역 측정·보고.
- **수정** `apps/desktop-tauri/src/floatbar/FloatBar.test.tsx` — 보고 동작 테스트.
- **수정** `apps/desktop-tauri/src/floatbar/SettingsSection.tsx` — "Click-Through" 라벨/설명 갱신.

테스트 실행 디렉터리: Rust는 `apps/desktop-tauri/src-tauri`, 프론트는 `apps/desktop-tauri`.

---

## Task 1: 순수 함수 + hit_test 모듈 스켈레톤

좌표 변환과 점-사각형 판정을 순수 함수로 만들고 단위 테스트한다.

**Files:**
- Create: `apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs`
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/mod.rs:8-13` (모듈 등록)

- [ ] **Step 1: hit_test.rs 생성 (순수 함수 + 테스트)**

`apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs`:

```rust
//! FloatBar 하이브리드 클릭 통과의 좌표 계산과 적중 판정.
//!
//! 프론트엔드가 보고한 바의 "창-상대 논리 사각형"을 화면 물리 사각형으로
//! 바꾸고, 커서가 그 안에 있는지 판정한다. 순수 함수로 분리해 OS·창 핸들
//! 없이 단위 테스트할 수 있게 한다.

/// 창 좌상단 기준 논리(CSS) 픽셀 사각형. 프론트엔드가 보고한다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// 화면(모니터) 기준 물리 픽셀 사각형. 커서 비교에 쓴다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// 창-상대 논리 사각형을 화면 물리 사각형으로 변환한다.
///
/// `win_pos`는 창의 물리 좌상단(`outer_position`), `scale`은 DPI 배율.
pub fn window_rect_to_screen(rect: HitRect, win_pos: (i32, i32), scale: f64) -> ScreenRect {
    ScreenRect {
        x: win_pos.0 as f64 + rect.x * scale,
        y: win_pos.1 as f64 + rect.y * scale,
        w: rect.w * scale,
        h: rect.h * scale,
    }
}

/// 물리 커서 좌표가 사각형 안(경계 포함)인지 판정한다.
pub fn point_in_rect(cursor: (f64, f64), rect: ScreenRect) -> bool {
    cursor.0 >= rect.x
        && cursor.0 <= rect.x + rect.w
        && cursor.1 >= rect.y
        && cursor.1 <= rect.y + rect.h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_to_screen_scale_one_adds_window_offset() {
        let rect = HitRect { x: 10.0, y: 10.0, w: 200.0, h: 40.0 };
        let screen = window_rect_to_screen(rect, (100, 100), 1.0);
        assert_eq!(screen, ScreenRect { x: 110.0, y: 110.0, w: 200.0, h: 40.0 });
    }

    #[test]
    fn rect_to_screen_scale_two_scales_size_and_offset() {
        let rect = HitRect { x: 10.0, y: 5.0, w: 100.0, h: 20.0 };
        let screen = window_rect_to_screen(rect, (200, 200), 2.0);
        // offset 10*2=20 -> 200+20=220, size 100*2=200.
        assert_eq!(screen, ScreenRect { x: 220.0, y: 210.0, w: 200.0, h: 40.0 });
    }

    #[test]
    fn point_inside_is_true() {
        let rect = ScreenRect { x: 100.0, y: 100.0, w: 50.0, h: 50.0 };
        assert!(point_in_rect((120.0, 120.0), rect));
    }

    #[test]
    fn point_on_edge_is_true() {
        let rect = ScreenRect { x: 100.0, y: 100.0, w: 50.0, h: 50.0 };
        assert!(point_in_rect((100.0, 100.0), rect));
        assert!(point_in_rect((150.0, 150.0), rect));
    }

    #[test]
    fn point_outside_is_false() {
        let rect = ScreenRect { x: 100.0, y: 100.0, w: 50.0, h: 50.0 };
        assert!(!point_in_rect((99.0, 120.0), rect));
        assert!(!point_in_rect((120.0, 151.0), rect));
    }
}
```

- [ ] **Step 2: mod.rs에 모듈 등록**

`apps/desktop-tauri/src-tauri/src/floatbar/mod.rs`의 모듈 선언부(8-13행)를 다음으로 교체:

```rust
mod commands;
pub(crate) mod hit_test;
pub(crate) mod window;

pub use commands::*;
pub use window::FLOAT_BAR_CONFIG_CHANGED_EVENT;
pub use window::FLOATBAR_LABEL;
```

- [ ] **Step 3: 테스트 실행 (통과 확인)**

Run: `cd apps/desktop-tauri/src-tauri && cargo test --lib hit_test`
Expected: PASS (5 tests: rect_to_screen_*, point_*).

- [ ] **Step 4: Commit**

```bash
git add apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs apps/desktop-tauri/src-tauri/src/floatbar/mod.rs
git commit -m "feat(floatbar): add hit-test coordinate helpers"
```

---

## Task 2: 공유 상태 + set_float_bar_hit_rect command

프론트가 보고하는 바 영역을 저장할 상태와 command를 만든다.

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs` (상태 추가)
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/mod.rs` (재노출)
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/commands.rs` (command)
- Modify: `apps/desktop-tauri/src-tauri/src/main.rs:71` (manage), `:162` (handler)

- [ ] **Step 1: hit_test.rs에 공유 상태 추가**

`hit_test.rs` 상단의 `HitRect` 정의 위에 import를 추가하고, `point_in_rect` 함수 아래(테스트 모듈 위)에 상태 타입을 추가한다.

파일 맨 위(`//! ...` 주석 바로 다음)에 추가:

```rust
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
```

`point_in_rect` 함수와 `#[cfg(test)]` 사이에 추가:

```rust
/// 폴링 루프와 command가 공유하는 상태.
///
/// `Arc` 뉴타입이라 Tauri managed state로 등록하면서도 폴링 스레드로 값을
/// 복제(Arc clone)해 넘길 수 있다.
#[derive(Clone, Default)]
pub struct FloatBarHitState(pub Arc<FloatBarHitInner>);

#[derive(Default)]
pub struct FloatBarHitInner {
    /// 프론트가 보고한 바의 창-상대 논리 사각형. 미보고면 `None`.
    pub rect: Mutex<Option<HitRect>>,
    /// 폴링 루프 가동 여부 겸 정지 신호.
    pub poll_running: AtomicBool,
    /// 마지막으로 적용한 ignore 값(중복 토글 방지).
    pub last_ignore: AtomicBool,
}
```

- [ ] **Step 2: mod.rs에서 상태 재노출**

`apps/desktop-tauri/src-tauri/src/floatbar/mod.rs`의 `pub use window::FLOATBAR_LABEL;` 다음 줄에 추가:

```rust
pub use hit_test::FloatBarHitState;
```

- [ ] **Step 3: command 추가**

`apps/desktop-tauri/src-tauri/src/floatbar/commands.rs`의 import에 `State`가 이미 있는지 확인한다(`use tauri::{Emitter, Manager, State};` — 이미 존재). 파일 맨 끝(`set_float_bar_orientation` command 다음)에 추가:

```rust
/// 프론트엔드가 측정한 바의 창-상대 논리 사각형을 저장한다. 폴링 루프가
/// 이 값을 읽어 클릭 통과 영역을 판정한다.
#[tauri::command]
pub fn set_float_bar_hit_rect(
    state: State<'_, crate::floatbar::FloatBarHitState>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Result<(), String> {
    *state.0.rect.lock().unwrap() = Some(super::hit_test::HitRect { x, y, w, h });
    Ok(())
}
```

- [ ] **Step 4: main.rs에 managed state + handler 등록**

`apps/desktop-tauri/src-tauri/src/main.rs:71` `.manage(Mutex::new(initial_state))` 다음 줄에 추가:

```rust
        .manage(floatbar::FloatBarHitState::default())
```

`apps/desktop-tauri/src-tauri/src/main.rs:162` `floatbar::set_float_bar_orientation,` 다음 줄(handler 목록 안)에 추가:

```rust
            floatbar::set_float_bar_hit_rect,
```

- [ ] **Step 5: 컴파일 확인**

Run: `cd apps/desktop-tauri/src-tauri && cargo build`
Expected: 성공(경고 가능 — `poll_running`/`last_ignore` 미사용 경고는 Task 3에서 해소).

- [ ] **Step 6: Commit**

```bash
git add apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs apps/desktop-tauri/src-tauri/src/floatbar/mod.rs apps/desktop-tauri/src-tauri/src/floatbar/commands.rs apps/desktop-tauri/src-tauri/src/main.rs
git commit -m "feat(floatbar): add hit-rect state and report command"
```

---

## Task 3: 폴링 루프 + 모드 적용 헬퍼

커서를 읽어 ignore를 토글하는 루프와, 모드를 적용하는 헬퍼를 만든다.

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs`

- [ ] **Step 1: 커서 읽기 + 루프 + 헬퍼 추가**

`hit_test.rs`의 import 줄을 다음으로 교체(`Ordering`, `Duration`, Tauri 타입 추가):

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewWindow};
```

`FloatBarHitInner` 정의 아래(테스트 모듈 위)에 추가:

```rust
const POLL_INTERVAL_MS: u64 = 80;

/// Win32 `GetCursorPos`로 물리 커서 좌표를 읽는다. 실패 시 `None`.
#[cfg(windows)]
fn get_cursor_pos() -> Option<(f64, f64)> {
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetCursorPos(point: *mut Point) -> i32;
    }
    let mut p = Point { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut p) };
    if ok != 0 {
        Some((p.x as f64, p.y as f64))
    } else {
        None
    }
}

/// ignore 상태를 적용하되 직전 값과 다를 때만 창에 반영한다.
fn set_ignore(window: &WebviewWindow, ignore: bool, inner: &FloatBarHitInner) {
    if inner.last_ignore.swap(ignore, Ordering::SeqCst) != ignore {
        let _ = window.set_ignore_cursor_events(ignore);
    }
}

/// 하이브리드 폴링을 시작한다. 이미 가동 중이면 아무것도 하지 않는다.
/// Windows 전용 — 다른 OS에서는 호출되지 않는다.
#[cfg(windows)]
fn start_poll(app: &AppHandle) {
    let Some(state) = app.try_state::<FloatBarHitState>() else {
        return;
    };
    let inner = state.0.clone();
    if inner.poll_running.swap(true, Ordering::SeqCst) {
        return; // 이미 가동 중.
    }
    // last_ignore를 알 수 없는 값으로 초기화해 첫 틱에 반드시 한 번 적용되게 한다.
    inner.last_ignore.store(true, Ordering::SeqCst);

    let app = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
            if !inner.poll_running.load(Ordering::SeqCst) {
                break;
            }
            let Some(window) = app.get_webview_window(crate::floatbar::FLOATBAR_LABEL) else {
                inner.poll_running.store(false, Ordering::SeqCst);
                break;
            };
            let rect = *inner.rect.lock().unwrap();
            let Some(rect) = rect else {
                // 아직 영역 미보고 — 통과 상태 유지.
                set_ignore(&window, true, &inner);
                continue;
            };
            let Ok(pos) = window.outer_position() else {
                continue;
            };
            let scale = window.scale_factor().unwrap_or(1.0);
            let screen = window_rect_to_screen(rect, (pos.x, pos.y), scale);
            let Some(cursor) = get_cursor_pos() else {
                continue;
            };
            let inside = point_in_rect(cursor, screen);
            // 바 안이면 클릭 받기(ignore=false), 밖이면 통과(ignore=true).
            set_ignore(&window, !inside, &inner);
        }
    });
}

/// 폴링을 멈춘다. 루프는 다음 틱에 스스로 종료한다.
fn stop_poll(app: &AppHandle) {
    if let Some(state) = app.try_state::<FloatBarHitState>() {
        state.0.poll_running.store(false, Ordering::SeqCst);
    }
}

/// 클릭 통과 모드를 창에 적용한다.
///
/// - `click_through == true` (완전 통과): 폴링 정지 후 창 전체 통과.
/// - `click_through == false` (하이브리드): Windows에서는 통과로 시작한 뒤
///   폴링을 켜 바 위에서만 클릭을 받게 한다. 다른 OS에서는 폴링이 없으므로
///   창 전체를 상호작용 가능 상태로 둔다(기존 동작).
pub fn apply_click_through_mode(app: &AppHandle, window: &WebviewWindow, click_through: bool) {
    if click_through {
        stop_poll(app);
        let _ = window.set_ignore_cursor_events(true);
    } else {
        #[cfg(windows)]
        {
            let _ = window.set_ignore_cursor_events(true);
            start_poll(app);
        }
        #[cfg(not(windows))]
        {
            let _ = window.set_ignore_cursor_events(false);
        }
    }
}
```

- [ ] **Step 2: 컴파일 확인 (Windows)**

Run: `cd apps/desktop-tauri/src-tauri && cargo build`
Expected: 성공. (`apply_click_through_mode` 미사용 경고는 Task 4에서 호출하며 해소.)

- [ ] **Step 3: 기존 단위 테스트 재실행**

Run: `cd apps/desktop-tauri/src-tauri && cargo test --lib hit_test`
Expected: PASS (Task 1의 5 tests 그대로 통과 — 순수 함수 미변경).

- [ ] **Step 4: Commit**

```bash
git add apps/desktop-tauri/src-tauri/src/floatbar/hit_test.rs
git commit -m "feat(floatbar): add cursor polling loop and mode helper"
```

---

## Task 4: 모드 배선 (window.rs / commands.rs / mod.rs)

기존 `apply_click_through` 호출을 새 헬퍼로 교체하고, 미사용 함수를 제거한다.

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/window.rs:53,116` (호출 교체), `:240-269` (함수 제거)
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/commands.rs:45-55`
- Modify: `apps/desktop-tauri/src-tauri/src/floatbar/mod.rs:121-124`

- [ ] **Step 1: window.rs `show()` 내부 호출 교체**

`window.rs:53` `apply_click_through(&window, click_through);` 를 다음으로 교체:

```rust
        crate::floatbar::hit_test::apply_click_through_mode(app, &window, click_through);
```

`window.rs:116` `apply_click_through(&win, click_through);` 를 다음으로 교체:

```rust
    crate::floatbar::hit_test::apply_click_through_mode(app, &win, click_through);
```

- [ ] **Step 2: window.rs의 미사용 `apply_click_through` 제거**

`window.rs`의 `apply_click_through` 함수 전체(독 코멘트 `/// Toggle click-through ...`부터 함수 닫는 `}`까지, 현재 240-269행)를 삭제한다. (extern 블록과 `apply_opacity`는 그대로 둔다.)

삭제 후 `WS_EX_TRANSPARENT` 상수는 더 이상 쓰이지 않으므로, 같이 삭제된 함수 안에만 있었다면 추가 작업 없음(해당 상수는 함수 내부 지역 상수였음).

- [ ] **Step 3: commands.rs `set_float_bar_click_through` 교체**

`commands.rs:45-55`의 `set_float_bar_click_through` 함수를 다음으로 교체:

```rust
#[tauri::command]
pub fn set_float_bar_click_through(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = Settings::load();
    settings.float_bar_click_through = enabled;
    settings.save().map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window(floatbar_window::FLOATBAR_LABEL) {
        crate::floatbar::hit_test::apply_click_through_mode(&app, &window, enabled);
    }
    Ok(())
}
```

- [ ] **Step 4: mod.rs `apply_state` 재적용 분기 교체**

`mod.rs:121-124`의 `else if let Some(w) = ...` 분기를 다음으로 교체:

```rust
    } else if let Some(w) = app.get_webview_window(FLOATBAR_LABEL) {
        window::apply_opacity(&w, settings.float_bar_opacity);
        hit_test::apply_click_through_mode(app, &w, settings.float_bar_click_through);
    }
```

- [ ] **Step 5: 컴파일 + Rust 테스트 전체 실행**

Run: `cd apps/desktop-tauri/src-tauri && cargo build && cargo test --lib`
Expected: 성공, 기존 테스트 + hit_test 테스트 모두 PASS. (`apply_click_through` 제거로 인한 미사용 경고 없어야 함.)

- [ ] **Step 6: Commit**

```bash
git add apps/desktop-tauri/src-tauri/src/floatbar/window.rs apps/desktop-tauri/src-tauri/src/floatbar/commands.rs apps/desktop-tauri/src-tauri/src/floatbar/mod.rs
git commit -m "refactor(floatbar): route click-through through hybrid mode helper"
```

---

## Task 5: 프론트엔드 바 영역 리포터

`FloatBar`가 바의 사각형을 측정해 command로 보고한다.

**Files:**
- Modify: `apps/desktop-tauri/src/floatbar/api.ts`
- Modify: `apps/desktop-tauri/src/floatbar/FloatBar.tsx`
- Modify: `apps/desktop-tauri/src/floatbar/FloatBar.test.tsx`

- [ ] **Step 1: 실패하는 테스트 작성**

`FloatBar.test.tsx`의 `describe("FloatBar", ...)` 블록 안, 첫 번째 `it(...)` 앞에 ResizeObserver 목을 추가한다. `beforeEach` 블록을 다음으로 교체:

```typescript
  beforeEach(() => {
    eventListeners.clear();
    vi.clearAllMocks();
    // jsdom에는 ResizeObserver가 없으므로 목으로 채운다(콜백 미발화).
    vi.stubGlobal(
      "ResizeObserver",
      class {
        observe() {}
        unobserve() {}
        disconnect() {}
      },
    );
  });
```

그리고 새 테스트를 `describe` 블록 끝(마지막 `it` 다음)에 추가:

```typescript
  it("reports the bar hit-rect on mount", async () => {
    render(<FloatBar state={bootstrap()} />);
    await Promise.resolve();

    const hitRectCalls = coreMocks.invoke.mock.calls.filter(
      ([command]) => command === "set_float_bar_hit_rect",
    );
    expect(hitRectCalls.length).toBeGreaterThanOrEqual(1);
    expect(hitRectCalls[0][1]).toEqual(
      expect.objectContaining({
        x: expect.any(Number),
        y: expect.any(Number),
        w: expect.any(Number),
        h: expect.any(Number),
      }),
    );
  });
```

- [ ] **Step 2: 테스트 실행 (실패 확인)**

Run: `cd apps/desktop-tauri && pnpm test -- FloatBar`
Expected: FAIL — "reports the bar hit-rect on mount"에서 `set_float_bar_hit_rect` 호출이 0건.

- [ ] **Step 3: api.ts에 래퍼 추가**

`apps/desktop-tauri/src/floatbar/api.ts`의 `setFloatBarOrientation` 함수 다음에 추가:

```typescript
export interface FloatBarHitRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export function setFloatBarHitRect(rect: FloatBarHitRect): Promise<void> {
  return invoke<void>("set_float_bar_hit_rect", rect);
}
```

- [ ] **Step 4: FloatBar.tsx에서 측정·보고**

`FloatBar.tsx` 상단 import에 `setFloatBarHitRect`를 추가한다. 7행 `import { useDragOrClick } from "./useDragOrClick";` 다음에 추가:

```typescript
import { setFloatBarHitRect } from "./api";
```

`useRef` 기반 바 ref를 추가한다. `const openTimerRef = useRef<number | null>(null);` (44행) 다음에 추가:

```typescript
  const barRef = useRef<HTMLDivElement | null>(null);
```

바 영역을 측정·보고하는 effect를 추가한다. body 클래스 effect(46-53행) 다음에 추가:

```typescript
  useEffect(() => {
    const node = barRef.current;
    if (!node) {
      return;
    }
    const report = () => {
      const r = node.getBoundingClientRect();
      void setFloatBarHitRect({ x: r.x, y: r.y, w: r.width, h: r.height }).catch(() => {});
    };
    report();
    const observer = new ResizeObserver(report);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);
```

루트 `.floatbar` div에 ref를 단다. `<div className="floatbar"` 가 있는 100-105행의 여는 태그를 다음으로 교체:

```typescript
    <div
      ref={barRef}
      className="floatbar"
      onContextMenu={(event) => event.preventDefault()}
      {...dragOrClick}
    >
```

- [ ] **Step 5: 테스트 실행 (통과 확인)**

Run: `cd apps/desktop-tauri && pnpm test -- FloatBar`
Expected: PASS — 신규 테스트 포함 모든 FloatBar 테스트 통과.

- [ ] **Step 6: Commit**

```bash
git add apps/desktop-tauri/src/floatbar/api.ts apps/desktop-tauri/src/floatbar/FloatBar.tsx apps/desktop-tauri/src/floatbar/FloatBar.test.tsx
git commit -m "feat(floatbar): report bar hit-rect from frontend"
```

---

## Task 6: Click-Through 토글 라벨/설명 갱신

토글이 "OFF=하이브리드, ON=완전통과"임을 사용자에게 알린다.

**Files:**
- Modify: `apps/desktop-tauri/src/floatbar/SettingsSection.tsx:75-85`

- [ ] **Step 1: 라벨·설명 교체**

`SettingsSection.tsx:75-85`의 `<Field label="Click-Through" ...>` 블록을 다음으로 교체:

```tsx
        <Field
          label="Full Click-Through"
          description="On: the whole bar passes clicks through. Off: only the bar itself is clickable; the empty area around it passes through to the window underneath."
          leading
        >
          <Toggle
            checked={settings.floatBarClickThrough}
            disabled={saving || !settings.floatBarEnabled}
            onChange={(v) => set({ floatBarClickThrough: v })}
          />
        </Field>
```

- [ ] **Step 2: 프론트 테스트 전체 실행 (회귀 확인)**

Run: `cd apps/desktop-tauri && pnpm test`
Expected: PASS — "Click-Through" 문자열에 의존하는 테스트가 없는지 확인. 실패하면 해당 테스트의 기대 문자열을 "Full Click-Through"로 갱신.

- [ ] **Step 3: Commit**

```bash
git add apps/desktop-tauri/src/floatbar/SettingsSection.tsx
git commit -m "docs(floatbar): clarify click-through toggle label for hybrid mode"
```

---

## Task 7: tray "설정 열기" 진입로

tray 메뉴에 Settings 창(Display 탭)을 여는 항목을 추가한다.

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/tray_menu.rs:74-82` (항목), `:168-180` (테스트)
- Modify: `apps/desktop-tauri/src-tauri/src/tray_bridge.rs:153-168` (액션), `:234-274` (핸들)

- [ ] **Step 1: 실패하는 테스트 작성 (tray_menu.rs)**

`tray_menu.rs`의 `proof_menu_items_show_simplified_entries` 테스트(168-180행)의 `assert_eq!` 기대값을 다음으로 교체:

```rust
        assert_eq!(
            items,
            vec![
                "Detail 열기",
                "Bar 표시 토글",
                "Watchdog 재시도",
                "설정 열기",
                "종료"
            ]
        );
```

- [ ] **Step 2: 테스트 실행 (실패 확인)**

Run: `cd apps/desktop-tauri/src-tauri && cargo test --lib proof_menu_items_show_simplified_entries`
Expected: FAIL — 현재 메뉴에 "설정 열기"가 없어 vec 불일치.

- [ ] **Step 3: tray_menu.rs에 항목 추가**

`tray_menu.rs:74-81`의 `vec![...]`를 다음으로 교체(separator 앞에 "설정 열기" 삽입):

```rust
    vec![
        TrayMenuEntry::item("toggle_detail", "Detail 열기"),
        TrayMenuEntry::item("toggle_bar_visibility", "Bar 표시 토글"),
        TrayMenuEntry::item("watchdog_retry", "Watchdog 재시도")
            .with_disabled(bar_state != BarState::Paused),
        TrayMenuEntry::item("open_settings", "설정 열기"),
        TrayMenuEntry::separator(),
        TrayMenuEntry::item("quit", "종료"),
    ]
```

- [ ] **Step 4: 테스트 실행 (통과 확인)**

Run: `cd apps/desktop-tauri/src-tauri && cargo test --lib proof_menu_items_show_simplified_entries`
Expected: PASS.

- [ ] **Step 5: tray_bridge.rs에 액션 추가**

`tray_bridge.rs:153-158`의 `enum MenuAction`에 변형 추가:

```rust
enum MenuAction {
    ToggleDetail,
    ToggleBarVisibility,
    WatchdogRetry,
    OpenSettings,
    Quit,
}
```

`tray_bridge.rs:160-168`의 `resolve_menu_action`에 매핑 추가(`"watchdog_retry"` 다음 줄):

```rust
        "open_settings" => Some(MenuAction::OpenSettings),
```

`tray_bridge.rs:234-274`의 `handle_menu_event`에서 `Some(MenuAction::WatchdogRetry) => { ... }` 블록 다음에 추가:

```rust
        Some(MenuAction::OpenSettings) => {
            let _ = shell::settings_window::open_or_focus(app, "display");
        }
```

- [ ] **Step 6: resolve_menu_action 테스트 추가 (tray_bridge.rs)**

`tray_bridge.rs`의 기존 테스트 모듈에서 `resolve_menu_action("toggle_detail")` 류 단언이 있는 테스트(601-610행 부근)를 찾아, 그 테스트 함수 안에 다음 단언을 추가한다:

```rust
        assert!(matches!(
            resolve_menu_action("open_settings"),
            Some(MenuAction::OpenSettings)
        ));
```

(주: `MenuAction`은 `PartialEq`가 없을 수 있으므로 `matches!`로 비교한다.)

- [ ] **Step 7: 컴파일 + tray 테스트 실행**

Run: `cd apps/desktop-tauri/src-tauri && cargo test --lib tray`
Expected: PASS — tray_menu / tray_bridge 테스트 모두 통과.

- [ ] **Step 8: Commit**

```bash
git add apps/desktop-tauri/src-tauri/src/tray_menu.rs apps/desktop-tauri/src-tauri/src/tray_bridge.rs
git commit -m "feat(tray): add Open Settings entry to tray menu"
```

---

## Task 8: 수동 검증 (Manual Verification)

빌드해 실제 동작을 확인한다. (자동 테스트로 못 잡는 OS 상호작용.)

**Files:** 없음 (실행만)

- [ ] **Step 1: 앱 실행**

Run: `cd apps/desktop-tauri && pnpm run tauri:dev`

- [ ] **Step 2: 하이브리드(기본) 검증**

- FloatBar가 뜬 상태에서 **바 위 클릭** → detail 토글 동작 확인.
- 바 **주변 빈 영역 클릭** → 뒤에 있는 창/바탕화면이 반응(클릭 통과) 확인.
- 바를 **드래그로 이동**한 뒤 위 두 동작이 그대로 성립하는지 확인.

- [ ] **Step 3: 설정 진입로 + 완전 통과 검증**

- tray 우클릭 → **"설정 열기"** → Settings 창의 **Display 탭**이 열리는지 확인.
- "Full Click-Through" 토글 **ON** → 바 포함 창 전체가 통과되는지 확인.
- 토글 **OFF** → 하이브리드로 복귀(바만 클릭) 확인.

- [ ] **Step 4: 검증 결과 기록**

검증 통과 시 사용자에게 보고. 실패 항목이 있으면 어느 Task를 재검토할지 함께 보고.

---

## Self-Review Notes

- **Spec coverage:** 모드 정의(Task 4·6), 폴링 메커니즘(Task 3), 컴포넌트 1~7(Task 1~7), 순수 함수 테스트(Task 1), set_ignore_cursor_events 통일(Task 4), tray 진입로(Task 7), 엣지 케이스(드래그/중복방지/모드전환은 Task 3 루프와 `poll_running` 플래그로 커버, hit rect 미보고는 루프의 `None` 분기로 커버). 모든 spec 요구가 Task에 매핑됨.
- **Type 일관성:** `HitRect`/`ScreenRect`/`FloatBarHitState`/`FloatBarHitInner`/`apply_click_through_mode`/`set_float_bar_hit_rect`/`setFloatBarHitRect` 이름이 모든 Task에서 동일.
- **No placeholders:** 모든 코드 스텝에 실제 코드 포함.
