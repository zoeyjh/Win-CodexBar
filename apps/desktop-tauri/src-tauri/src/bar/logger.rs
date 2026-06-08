use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Duration, Local, Utc};
use tokio::sync::broadcast;

use super::lifecycle_event::{LifecycleEvent, TimestampedEvent};
use super::{lifecycle_log_dir, state::BarRuntimeState};

const HEALTHY_WATCHDOG_THROTTLE_MINUTES: i64 = 5;
const LOG_RETENTION_DAYS: i64 = 7;

pub fn spawn(runtime: BarRuntimeState) {
    rotate_logs();

    let mut rx = runtime.subscribe();
    tauri::async_runtime::spawn(async move {
        let mut last_healthy_tick_at: Option<DateTime<Utc>> = None;

        loop {
            match rx.recv().await {
                Ok(event) => {
                    if !should_write_event(&event, &mut last_healthy_tick_at) {
                        continue;
                    }

                    if let Err(err) = append_event(&event) {
                        tracing::warn!(target: "codexbar::bar::logger", %err, "failed to append lifecycle event");
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!(target: "codexbar::bar::logger", skipped, "lifecycle logger lagged behind");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn append_event(event: &TimestampedEvent) -> Result<(), String> {
    let path = log_path_for(event.ts);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| err.to_string())?;
    let line = serde_json::to_string(event).map_err(|err| err.to_string())?;
    writeln!(file, "{line}").map_err(|err| err.to_string())
}

fn log_path_for(ts: DateTime<Utc>) -> PathBuf {
    lifecycle_log_dir().join(format!(
        "lifecycle-{}.log",
        ts.with_timezone(&Local).format("%Y-%m-%d")
    ))
}

fn rotate_logs() {
    let log_dir = lifecycle_log_dir();
    let Ok(entries) = fs::read_dir(&log_dir) else {
        return;
    };
    let cutoff = std::time::SystemTime::now().checked_sub(std::time::Duration::from_secs(
        (LOG_RETENTION_DAYS as u64) * 24 * 60 * 60,
    ));

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("lifecycle-") || !file_name.ends_with(".log") {
            continue;
        }

        let should_delete = cutoff
            .and_then(|cutoff| {
                entry
                    .metadata()
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .map(|modified| modified < cutoff)
            })
            .unwrap_or(false);
        if should_delete {
            let _ = fs::remove_file(path);
        }
    }
}

fn should_write_event(
    event: &TimestampedEvent,
    last_healthy_tick_at: &mut Option<DateTime<Utc>>,
) -> bool {
    match event.event {
        LifecycleEvent::WatchdogTick { healthy: true } => {
            if let Some(previous) = last_healthy_tick_at.as_ref()
                && event.ts - previous.to_owned()
                    < Duration::minutes(HEALTHY_WATCHDOG_THROTTLE_MINUTES)
            {
                return false;
            }
            *last_healthy_tick_at = Some(event.ts);
            true
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bar::lifecycle_event::TimestampedEvent;

    #[test]
    fn healthy_watchdog_events_are_throttled() {
        let mut last = None;
        let first = TimestampedEvent::now(LifecycleEvent::WatchdogTick { healthy: true });
        assert!(should_write_event(&first, &mut last));

        let second = TimestampedEvent {
            ts: first.ts + Duration::minutes(1),
            event: LifecycleEvent::WatchdogTick { healthy: true },
        };
        assert!(!should_write_event(&second, &mut last));

        let third = TimestampedEvent {
            ts: first.ts + Duration::minutes(HEALTHY_WATCHDOG_THROTTLE_MINUTES),
            event: LifecycleEvent::WatchdogTick { healthy: true },
        };
        assert!(should_write_event(&third, &mut last));
    }

    #[test]
    fn unhealthy_watchdog_events_are_not_throttled() {
        let mut last = None;
        let event = TimestampedEvent::now(LifecycleEvent::WatchdogTick { healthy: false });
        assert!(should_write_event(&event, &mut last));
        assert!(last.is_none());
    }
}
