use std::time::Duration;

use chrono::Utc;

#[path = "../src/bar/backoff.rs"]
mod backoff;
#[path = "../src/bar/lifecycle_event.rs"]
mod lifecycle_event;
#[path = "../src/bar/state.rs"]
mod state;
#[path = "../src/bar/watchdog.rs"]
mod watchdog;
#[path = "../src/bar/window_state.rs"]
mod window_state;

use backoff::BackoffPolicy;
use lifecycle_event::{HideReason, LifecycleEvent, TimestampedEvent};
use state::{BarCommand, BarState};
use watchdog::{WatchdogState, WindowHealthChecker, is_within_work_areas};
use window_state::{WindowRect, WindowState};

#[test]
fn bar_state_applies_valid_transitions() {
    assert_eq!(
        BarState::Visible.transition(BarCommand::Hide),
        BarState::IntentionallyHidden
    );
    assert_eq!(
        BarState::Visible.transition(BarCommand::StartRecovery),
        BarState::Recovering
    );
    assert_eq!(
        BarState::Visible.transition(BarCommand::Quit),
        BarState::Quitting
    );
    assert_eq!(
        BarState::IntentionallyHidden.transition(BarCommand::Show),
        BarState::Visible
    );
    assert_eq!(
        BarState::IntentionallyHidden.transition(BarCommand::Quit),
        BarState::Quitting
    );
    assert_eq!(
        BarState::Recovering.transition(BarCommand::RecoverySuccess),
        BarState::Visible
    );
    assert_eq!(
        BarState::Recovering.transition(BarCommand::RecoveryFailed {
            consecutive_failures: 4,
        }),
        BarState::Recovering,
    );
    assert_eq!(
        BarState::Recovering.transition(BarCommand::RecoveryFailed {
            consecutive_failures: 5,
        }),
        BarState::Paused,
    );
    assert_eq!(
        BarState::Recovering.transition(BarCommand::Quit),
        BarState::Quitting
    );
    assert_eq!(
        BarState::Paused.transition(BarCommand::Retry),
        BarState::Visible
    );
    assert_eq!(
        BarState::Paused.transition(BarCommand::Quit),
        BarState::Quitting
    );
}

#[test]
fn bar_state_ignores_invalid_transitions() {
    assert_eq!(
        BarState::Visible.transition(BarCommand::Show),
        BarState::Visible
    );
    assert_eq!(
        BarState::IntentionallyHidden.transition(BarCommand::StartRecovery),
        BarState::IntentionallyHidden,
    );
    assert_eq!(
        BarState::Recovering.transition(BarCommand::Hide),
        BarState::Recovering
    );
    assert_eq!(
        BarState::Paused.transition(BarCommand::Show),
        BarState::Paused
    );
    assert_eq!(
        BarState::Quitting.transition(BarCommand::Retry),
        BarState::Quitting
    );
}

#[test]
fn bar_state_reports_when_watchdog_is_active() {
    assert!(BarState::Visible.watchdog_active());
    assert!(!BarState::IntentionallyHidden.watchdog_active());
    assert!(!BarState::Recovering.watchdog_active());
    assert!(!BarState::Paused.watchdog_active());
    assert!(!BarState::Quitting.watchdog_active());
}

#[test]
fn timestamped_event_now_sets_timestamp_and_serializes_event_shape() {
    let event = TimestampedEvent::now(LifecycleEvent::Hidden {
        reason: HideReason::UserToggle,
    });

    assert!(event.ts <= Utc::now());

    let value = serde_json::to_value(&event).expect("event should serialize");
    assert_eq!(value["event"], "hidden");
    assert_eq!(value["reason"], "user_toggle");
}

#[test]
fn backoff_policy_uses_base_growth_cap_and_reset() {
    let mut policy = BackoffPolicy::new(Duration::from_secs(1), Duration::from_secs(5), 2.0, 0.0);

    assert_eq!(policy.next_delay(), Duration::from_secs(1));
    assert_eq!(policy.next_delay(), Duration::from_secs(2));
    assert_eq!(policy.next_delay(), Duration::from_secs(4));
    assert_eq!(policy.next_delay(), Duration::from_secs(5));
    assert_eq!(policy.next_delay(), Duration::from_secs(5));

    policy.reset();

    assert_eq!(policy.next_delay(), Duration::from_secs(1));
}

#[test]
fn window_state_returns_default_for_missing_file() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("missing.json");

    let loaded = WindowState::load(&path);

    assert_eq!(loaded.rect.x, 1600);
    assert_eq!(loaded.rect.y, 8);
    assert_eq!(loaded.rect.w, 300);
    assert_eq!(loaded.rect.h, 40);
    assert_eq!(loaded.coord_space, "logical");
}

#[test]
fn window_state_saves_and_loads_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let path = dir.path().join("nested").join("window-state.json");
    let state = WindowState {
        rect: WindowRect {
            x: 42,
            y: 16,
            w: 333,
            h: 44,
        },
        monitor_id: Some("monitor-1".to_string()),
        dpi: Some(1.25),
        saved_at: Utc::now(),
        coord_space: "logical".to_string(),
    };

    state.save(&path).expect("window state should save");
    let loaded = WindowState::load(&path);

    assert_eq!(loaded.rect.x, 42);
    assert_eq!(loaded.rect.y, 16);
    assert_eq!(loaded.rect.w, 333);
    assert_eq!(loaded.rect.h, 44);
    assert_eq!(loaded.monitor_id.as_deref(), Some("monitor-1"));
    assert_eq!(loaded.dpi, Some(1.25));
    assert_eq!(loaded.coord_space, "logical");
    assert_eq!(loaded.saved_at, state.saved_at);
}

#[test]
fn window_state_clamps_negative_coordinates_into_work_area() {
    let mut state = WindowState {
        rect: WindowRect {
            x: -100,
            y: -50,
            w: 300,
            h: 40,
        },
        ..WindowState::default_position()
    };

    state.clamp_to_work_area((0, 0, 1920, 1080));

    assert_eq!(state.rect.x, 0);
    assert_eq!(state.rect.y, 0);
}

#[test]
fn window_state_clamps_coordinates_past_work_area_edge() {
    let mut state = WindowState {
        rect: WindowRect {
            x: 1900,
            y: 1100,
            w: 300,
            h: 40,
        },
        ..WindowState::default_position()
    };

    state.clamp_to_work_area((0, 0, 1920, 1080));

    assert_eq!(state.rect.x, 1620);
    assert_eq!(state.rect.y, 1040);
}

#[derive(Clone)]
struct MockChecker {
    handle_valid: bool,
    visible: bool,
    always_on_top: Option<bool>,
    rect: Option<(i32, i32, u32, u32)>,
    work_areas: Vec<(i32, i32, u32, u32)>,
}

impl MockChecker {
    fn healthy() -> Self {
        Self {
            handle_valid: true,
            visible: true,
            always_on_top: Some(true),
            rect: Some((10, 10, 300, 40)),
            work_areas: vec![(0, 0, 1920, 1080)],
        }
    }
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
fn watchdog_healthy_tick_resets_failures() {
    let mut watchdog = WatchdogState::new();
    watchdog.consecutive_failures = 2;

    let command = watchdog.tick(&MockChecker::healthy());

    assert_eq!(command, None);
    assert_eq!(watchdog.consecutive_failures, 0);
}

#[test]
fn watchdog_starts_recovery_after_three_consecutive_failures() {
    let mut watchdog = WatchdogState::new();
    let checker = MockChecker {
        visible: false,
        ..MockChecker::healthy()
    };

    assert_eq!(watchdog.tick(&checker), None);
    assert_eq!(watchdog.tick(&checker), None);
    assert_eq!(watchdog.tick(&checker), Some(BarCommand::StartRecovery));
    assert_eq!(watchdog.consecutive_failures, 0);
    assert_eq!(watchdog.consecutive_recovery_failures, 1);
}

#[test]
fn watchdog_treats_off_screen_windows_as_unhealthy() {
    let mut watchdog = WatchdogState::new();
    let checker = MockChecker {
        rect: Some((3000, 10, 300, 40)),
        ..MockChecker::healthy()
    };

    assert_eq!(watchdog.tick(&checker), None);
    assert_eq!(watchdog.consecutive_failures, 1);
}

#[test]
fn watchdog_reports_work_area_boundary_membership() {
    let work_areas = vec![(0, 0, 1920, 1080)];

    assert!(is_within_work_areas((0, 0, 1920, 1080), &work_areas));
    assert!(is_within_work_areas((1620, 1040, 300, 40), &work_areas));
    assert!(!is_within_work_areas((1621, 1040, 300, 40), &work_areas));
    assert!(!is_within_work_areas((-1, 0, 300, 40), &work_areas));
}

#[test]
fn watchdog_recovery_commands_reflect_recovery_failures() {
    let mut watchdog = WatchdogState::new();
    let checker = MockChecker {
        handle_valid: false,
        ..MockChecker::healthy()
    };

    let _ = watchdog.tick(&checker);
    let _ = watchdog.tick(&checker);
    let _ = watchdog.tick(&checker);

    assert_eq!(
        watchdog.on_recovery_failed(),
        BarCommand::RecoveryFailed {
            consecutive_failures: 1,
        }
    );

    watchdog.on_recovery_success();
    assert_eq!(watchdog.consecutive_recovery_failures, 0);
}
