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

pub fn is_within_work_areas(
    pos: (i32, i32, u32, u32),
    work_areas: &[(i32, i32, u32, u32)],
) -> bool {
    let (wx, wy, ww, wh) = pos;
    work_areas.iter().any(|&(ax, ay, aw, ah)| {
        wx >= ax
            && wy >= ay
            && (wx + ww as i32) <= (ax + aw as i32)
            && (wy + wh as i32) <= (ay + ah as i32)
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
        let window = self.app.get_webview_window(crate::floatbar::FLOATBAR_LABEL)?;
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
        if !checker.is_handle_valid() {
            return false;
        }
        if !checker.is_visible() {
            return false;
        }
        if let Some(on_top) = checker.is_always_on_top()
            && !on_top
        {
            return false;
        }
        if let Some(rect) = checker.outer_rect() {
            let areas = checker.available_work_areas();
            if !areas.is_empty() && !is_within_work_areas(rect, &areas) {
                return false;
            }
        }
        true
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

            let recovery = watchdog.tick(&checker);
            let healthy = recovery.is_none() && watchdog.consecutive_failures == 0;
            runtime.emit(LifecycleEvent::WatchdogTick { healthy });

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

    #[test]
    fn window_rect_must_fit_within_a_work_area() {
        assert!(is_within_work_areas((10, 10, 20, 20), &[(0, 0, 100, 100)]));
        assert!(!is_within_work_areas((90, 90, 20, 20), &[(0, 0, 100, 100)]));
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
