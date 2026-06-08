//! FloatBar 하이브리드 클릭 통과의 좌표 계산과 적중 판정.
//!
//! 프론트엔드가 보고한 바의 "창-상대 논리 사각형"을 화면 물리 사각형으로
//! 바꾸고, 커서가 그 안에 있는지 판정한다. 순수 함수로 분리해 OS·창 핸들
//! 없이 단위 테스트할 수 있게 한다.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewWindow};

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
