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

impl WatchdogState {
    pub fn new() -> Self {
        Self {
            consecutive_failures: 0,
            consecutive_recovery_failures: 0,
        }
    }

    pub fn tick(&mut self, checker: &dyn WindowHealthChecker) -> Option<super::state::BarCommand> {
        if self.check_health(checker) {
            self.consecutive_failures = 0;
            None
        } else {
            self.consecutive_failures += 1;
            if self.consecutive_failures >= 3 {
                self.consecutive_failures = 0;
                self.consecutive_recovery_failures += 1;
                Some(super::state::BarCommand::StartRecovery)
            } else {
                None
            }
        }
    }

    pub fn on_recovery_success(&mut self) {
        self.consecutive_recovery_failures = 0;
    }

    pub fn on_recovery_failed(&mut self) -> super::state::BarCommand {
        super::state::BarCommand::RecoveryFailed {
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
        if let Some(on_top) = checker.is_always_on_top() {
            if !on_top {
                return false;
            }
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
