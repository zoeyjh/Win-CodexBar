use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BarState {
    Visible,
    IntentionallyHidden,
    Recovering,
    Paused,
    Quitting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarCommand {
    Show,
    Hide,
    StartRecovery,
    RecoverySuccess,
    RecoveryFailed { consecutive_failures: u32 },
    Retry,
    Quit,
}

impl BarState {
    pub fn transition(self, cmd: BarCommand) -> BarState {
        use BarCommand::*;
        use BarState::*;

        match (self, cmd) {
            (Visible, Hide) => IntentionallyHidden,
            (Visible, StartRecovery) => Recovering,
            (Visible, Quit) => Quitting,
            (IntentionallyHidden, Show) => Visible,
            (IntentionallyHidden, Quit) => Quitting,
            (Recovering, RecoverySuccess) => Visible,
            (
                Recovering,
                RecoveryFailed {
                    consecutive_failures,
                },
            ) if consecutive_failures >= 5 => Paused,
            (Recovering, RecoveryFailed { .. }) => Recovering,
            (Recovering, Quit) => Quitting,
            (Paused, Retry) => Visible,
            (Paused, Quit) => Quitting,
            _ => self,
        }
    }

    pub fn watchdog_active(self) -> bool {
        matches!(self, BarState::Visible)
    }
}
