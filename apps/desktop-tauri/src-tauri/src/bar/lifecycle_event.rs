use chrono::{DateTime, Utc};
use serde::Serialize;

use super::state::BarState;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum LifecycleEvent {
    Created { monitor: Option<u32>, pos: [i32; 2] },
    Shown,
    Hidden { reason: HideReason },
    Destroyed,
    AlwaysOnTopChanged { value: bool },
    FocusChanged { focused: bool },
    MonitorChanged { from: Option<u32>, to: u32 },
    StateTransition { from: BarState, to: BarState },
    WatchdogTick { healthy: bool },
    RecoveryAttempt { attempt: u32 },
    RecoverySuccess { attempt: u32 },
    RecoveryFailed { attempt: u32, reason: String },
    SkippedRecovery { reason: String },
    Error { message: String, context: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HideReason {
    UserToggle,
    Watchdog,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimestampedEvent {
    pub ts: DateTime<Utc>,
    #[serde(flatten)]
    pub event: LifecycleEvent,
}

impl TimestampedEvent {
    pub fn now(event: LifecycleEvent) -> Self {
        Self {
            ts: Utc::now(),
            event,
        }
    }
}
