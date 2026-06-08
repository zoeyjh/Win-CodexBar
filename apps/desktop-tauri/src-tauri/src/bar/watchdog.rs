use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::lifecycle_event::LifecycleEvent;
use super::state::{BarCommand, BarRuntimeState};

pub trait WindowHealthChecker: Send + Sync {
    fn is_handle_valid(&self) -> bool;
    fn is_visible(&self) -> bool;
    fn is_always_on_top(&self) -> Option<bool>;
    fn outer_rect(&self) -> Option<(i32, i32, u32, u32)>;
    fn available_work_areas(&self) -> Vec<(i32, i32, u32, u32)>;
}

/// True when the window rect overlaps at least one monitor work area.
///
/// The float bar is a tall, mostly-transparent strip the user can drag
/// anywhere; requiring it to fit *entirely* inside one monitor wrongly flags
/// a bar dropped low on a screen (its transparent bottom hangs off the edge)
/// or mid-drag across a monitor seam as unhealthy. We only treat the window
/// as lost when it overlaps *no* monitor at all — e.g. a display was
/// unplugged and the window is stranded in dead space.
pub fn intersects_any_work_area(
    pos: (i32, i32, u32, u32),
    work_areas: &[(i32, i32, u32, u32)],
) -> bool {
    let (wx, wy, ww, wh) = pos;
    let (wl, wt, wr, wb) = (wx, wy, wx + ww as i32, wy + wh as i32);
    work_areas.iter().any(|&(ax, ay, aw, ah)| {
        let (al, at, ar, ab) = (ax, ay, ax + aw as i32, ay + ah as i32);
        wl < ar && wr > al && wt < ab && wb > at
    })
}

pub struct WatchdogState {
    pub consecutive_failures: u32,
    pub consecutive_recovery_failures: u32,
}

#[cfg(not(test))]
pub struct TauriWindowHealthChecker {
    app: AppHandle,
}

#[cfg(not(test))]
impl TauriWindowHealthChecker {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[cfg(not(test))]
impl WindowHealthChecker for TauriWindowHealthChecker {
    fn is_handle_valid(&self) -> bool {
        self.app
            .get_webview_window(crate::floatbar::FLOATBAR_LABEL)
            .is_some()
    }

    fn is_visible(&self) -> bool {
        self.app
            .get_webview_window(crate::floatbar::FLOATBAR_LABEL)
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false)
    }

    fn is_always_on_top(&self) -> Option<bool> {
        None
    }

    fn outer_rect(&self) -> Option<(i32, i32, u32, u32)> {
        let window = self
            .app
            .get_webview_window(crate::floatbar::FLOATBAR_LABEL)?;
        let pos = window.outer_position().ok()?;
        let size = window.outer_size().ok()?;
        Some((pos.x, pos.y, size.width, size.height))
    }

    fn available_work_areas(&self) -> Vec<(i32, i32, u32, u32)> {
        self.app
            .get_webview_window(crate::floatbar::FLOATBAR_LABEL)
            .and_then(|window| window.available_monitors().ok())
            .unwrap_or_default()
            .into_iter()
            .map(|monitor| {
                let pos = monitor.position();
                let size = monitor.size();
                (pos.x, pos.y, size.width, size.height)
            })
            .collect()
    }
}

impl WatchdogState {
    pub fn new() -> Self {
        Self {
            consecutive_failures: 0,
            consecutive_recovery_failures: 0,
        }
    }

    pub fn tick(&mut self, checker: &dyn WindowHealthChecker) -> Option<BarCommand> {
        if self.check_health(checker) {
            self.consecutive_failures = 0;
            None
        } else {
            self.consecutive_failures += 1;
            if self.consecutive_failures >= 3 {
                self.consecutive_failures = 0;
                self.consecutive_recovery_failures += 1;
                Some(BarCommand::StartRecovery)
            } else {
                None
            }
        }
    }

    pub fn on_recovery_success(&mut self) {
        self.consecutive_recovery_failures = 0;
    }

    pub fn on_recovery_failed(&mut self) -> BarCommand {
        BarCommand::RecoveryFailed {
            consecutive_failures: self.consecutive_recovery_failures,
        }
    }

    fn check_health(&self, checker: &dyn WindowHealthChecker) -> bool {
        self.health_reason(checker).is_none()
    }

    /// Returns `None` when the window is healthy, or `Some(reason)` naming the
    /// first failing sub-check. Used both by [`check_health`] and by the
    /// diagnostic emit in [`spawn`].
    pub fn health_reason(&self, checker: &dyn WindowHealthChecker) -> Option<&'static str> {
        if !checker.is_handle_valid() {
            return Some("handle_invalid");
        }
        if !checker.is_visible() {
            return Some("not_visible");
        }
        if let Some(on_top) = checker.is_always_on_top()
            && !on_top
        {
            return Some("not_always_on_top");
        }
        if let Some(rect) = checker.outer_rect() {
            let areas = checker.available_work_areas();
            if !areas.is_empty() && !intersects_any_work_area(rect, &areas) {
                return Some("off_all_monitors");
            }
        }
        None
    }
}

#[cfg(not(test))]
pub fn spawn(app: AppHandle, runtime: BarRuntimeState) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut watchdog = WatchdogState::new();
        let checker = TauriWindowHealthChecker::new(app.clone());

        loop {
            interval.tick().await;

            if !runtime.current().watchdog_active() {
                watchdog.consecutive_failures = 0;
                continue;
            }

            // DIAGNOSTIC: capture the failing reason + geometry *before*
            // ticking so the lifecycle log shows why the check trips.
            let reason = watchdog.health_reason(&checker);

            let recovery = watchdog.tick(&checker);
            let healthy = recovery.is_none() && watchdog.consecutive_failures == 0;
            runtime.emit(LifecycleEvent::WatchdogTick { healthy });

            if let Some(reason) = reason {
                let rect = checker
                    .outer_rect()
                    .map(|(x, y, w, h)| [x, y, w as i32, h as i32]);
                let areas = checker
                    .available_work_areas()
                    .into_iter()
                    .map(|(x, y, w, h)| [x, y, w as i32, h as i32])
                    .collect();
                runtime.emit(LifecycleEvent::WatchdogUnhealthy {
                    reason: reason.to_string(),
                    rect,
                    areas,
                });
            }

            if let Some(command) = recovery {
                let _ = runtime.try_send(command);
            }
        }
    });
}

#[cfg(test)]
pub fn spawn(_app: AppHandle, _runtime: BarRuntimeState) {}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockChecker {
        handle_valid: bool,
        visible: bool,
        always_on_top: Option<bool>,
        rect: Option<(i32, i32, u32, u32)>,
        work_areas: Vec<(i32, i32, u32, u32)>,
    }

    impl WindowHealthChecker for MockChecker {
        fn is_handle_valid(&self) -> bool {
            self.handle_valid
        }

        fn is_visible(&self) -> bool {
            self.visible
        }

        fn is_always_on_top(&self) -> Option<bool> {
            self.always_on_top
        }

        fn outer_rect(&self) -> Option<(i32, i32, u32, u32)> {
            self.rect
        }

        fn available_work_areas(&self) -> Vec<(i32, i32, u32, u32)> {
            self.work_areas.clone()
        }
    }

    // Two side-by-side 1920x1080 monitors, matching the repro hardware.
    fn dual_monitors() -> Vec<(i32, i32, u32, u32)> {
        vec![(0, 0, 1920, 1080), (1920, 0, 1920, 1080)]
    }

    #[test]
    fn window_fully_inside_a_monitor_is_healthy() {
        assert!(intersects_any_work_area((10, 10, 20, 20), &[(0, 0, 100, 100)]));
    }

    #[test]
    fn bar_dropped_low_with_bottom_off_screen_stays_healthy() {
        // Repro: bar dropped low on the right monitor; its 220px-tall
        // (mostly transparent) window hangs 80px below the screen bottom.
        // It still overlaps the monitor, so it must NOT trigger recovery.
        assert!(intersects_any_work_area((3469, 944, 380, 220), &dual_monitors()));
    }

    #[test]
    fn bar_straddling_the_monitor_seam_stays_healthy() {
        // Repro: mid-drag across the x=1920 seam — overlaps both monitors.
        assert!(intersects_any_work_area((1711, 859, 380, 220), &dual_monitors()));
    }

    #[test]
    fn bar_stranded_off_all_monitors_is_unhealthy() {
        // A display was unplugged; the window sits in dead space with zero
        // overlap. This is the only case that should trigger recovery.
        assert!(!intersects_any_work_area((5000, 2000, 380, 220), &dual_monitors()));
    }

    #[test]
    fn watchdog_starts_recovery_after_three_failures() {
        let checker = MockChecker {
            handle_valid: false,
            visible: false,
            always_on_top: None,
            rect: None,
            work_areas: Vec::new(),
        };
        let mut watchdog = WatchdogState::new();
        assert!(watchdog.tick(&checker).is_none());
        assert!(watchdog.tick(&checker).is_none());
        assert_eq!(watchdog.tick(&checker), Some(BarCommand::StartRecovery));
    }

    #[test]
    fn watchdog_accepts_visible_window_inside_work_area() {
        let checker = MockChecker {
            handle_valid: true,
            visible: true,
            always_on_top: None,
            rect: Some((10, 10, 20, 20)),
            work_areas: vec![(0, 0, 100, 100)],
        };
        let mut watchdog = WatchdogState::new();
        assert!(watchdog.tick(&checker).is_none());
        assert_eq!(watchdog.consecutive_failures, 0);
    }
}
