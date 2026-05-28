use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

use codexbar::settings::Settings;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::{broadcast, mpsc};

use super::lifecycle_event::{LifecycleEvent, TimestampedEvent};

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

#[derive(Clone)]
pub struct BarRuntimeState {
    command_tx: mpsc::Sender<BarCommand>,
    lifecycle_tx: broadcast::Sender<TimestampedEvent>,
    current_state: Arc<RwLock<BarState>>,
}

impl BarRuntimeState {
    pub fn new(
        command_tx: mpsc::Sender<BarCommand>,
        lifecycle_tx: broadcast::Sender<TimestampedEvent>,
        initial_state: BarState,
    ) -> Self {
        Self {
            command_tx,
            lifecycle_tx,
            current_state: Arc::new(RwLock::new(initial_state)),
        }
    }

    pub async fn send(&self, cmd: BarCommand) -> Result<(), String> {
        self.command_tx.send(cmd).await.map_err(|err| err.to_string())
    }

    pub fn try_send(&self, cmd: BarCommand) -> Result<(), String> {
        self.command_tx.try_send(cmd).map_err(|err| err.to_string())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TimestampedEvent> {
        self.lifecycle_tx.subscribe()
    }

    pub fn emit(&self, event: LifecycleEvent) {
        let _ = self.lifecycle_tx.send(TimestampedEvent::now(event));
    }

    pub fn current(&self) -> BarState {
        *self.current_state.read().unwrap()
    }

    pub fn set_current(&self, next: BarState) {
        *self.current_state.write().unwrap() = next;
    }
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

#[cfg(not(test))]
fn rebuild_tray_menu(app: &AppHandle) {
    crate::tray_bridge::rebuild_tray_menu(app);
}

#[cfg(test)]
fn rebuild_tray_menu(_app: &AppHandle) {}

#[cfg(not(test))]
fn hide_bar_window(app: &AppHandle) -> Result<(), String> {
    crate::floatbar::hide_bar_window(app)
}

#[cfg(test)]
fn hide_bar_window(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(not(test))]
fn floatbar_label() -> &'static str {
    crate::floatbar::FLOATBAR_LABEL
}

#[cfg(test)]
fn floatbar_label() -> &'static str {
    "floatbar"
}

#[cfg(not(test))]
fn show_bar_window_inner(
    app: &AppHandle,
    opacity: u8,
    orientation: &str,
    click_through: bool,
) -> Result<(), String> {
    crate::floatbar::show_bar_window(app, opacity, orientation, click_through)
}

#[cfg(test)]
fn show_bar_window_inner(
    _app: &AppHandle,
    _opacity: u8,
    _orientation: &str,
    _click_through: bool,
) -> Result<(), String> {
    Ok(())
}

pub fn spawn_actor(app: AppHandle, runtime: BarRuntimeState, mut rx: mpsc::Receiver<BarCommand>) {
    tauri::async_runtime::spawn(async move {
        let mut current = BarState::Visible;
        let mut pending = VecDeque::new();
        let mut consecutive_recovery_failures = 0_u32;
        runtime.set_current(current);

        loop {
            let cmd = if let Some(cmd) = pending.pop_front() {
                cmd
            } else {
                match rx.recv().await {
                    Some(cmd) => cmd,
                    None => break,
                }
            };

            let next = current.transition(cmd);
            if next != current {
                runtime.set_current(next);
                runtime.emit(LifecycleEvent::StateTransition {
                    from: current,
                    to: next,
                });
                current = next;
                rebuild_tray_menu(&app);
            }

            match cmd {
                BarCommand::Show | BarCommand::Retry => {
                    if let Err(err) = show_bar_window(&app, &runtime) {
                        runtime.emit(LifecycleEvent::Error {
                            message: err,
                            context: "show_bar_window".into(),
                        });
                    }
                }
                BarCommand::Hide => {
                    if let Err(err) = hide_bar_window(&app) {
                        runtime.emit(LifecycleEvent::Error {
                            message: err,
                            context: "hide_bar_window".into(),
                        });
                    } else {
                        runtime.emit(LifecycleEvent::Hidden {
                            reason: super::lifecycle_event::HideReason::UserToggle,
                        });
                        runtime.emit(LifecycleEvent::Destroyed);
                    }
                }
                BarCommand::StartRecovery if current == BarState::Recovering => {
                    consecutive_recovery_failures = consecutive_recovery_failures.saturating_add(1);
                    let attempt = consecutive_recovery_failures;
                    runtime.emit(LifecycleEvent::RecoveryAttempt { attempt });
                    let _ = hide_bar_window(&app);
                    match show_bar_window(&app, &runtime) {
                        Ok(()) => {
                            runtime.emit(LifecycleEvent::RecoverySuccess { attempt });
                            pending.push_back(BarCommand::RecoverySuccess);
                        }
                        Err(reason) => {
                            runtime.emit(LifecycleEvent::RecoveryFailed {
                                attempt,
                                reason: reason.clone(),
                            });
                            pending.push_back(BarCommand::RecoveryFailed {
                                consecutive_failures: attempt,
                            });
                        }
                    }
                }
                BarCommand::RecoverySuccess => {
                    consecutive_recovery_failures = 0;
                }
                BarCommand::Quit => {
                    let _ = hide_bar_window(&app);
                }
                _ => {}
            }

            if current == BarState::Quitting {
                break;
            }
        }
    });
}

fn show_bar_window(app: &AppHandle, runtime: &BarRuntimeState) -> Result<(), String> {
    let settings = Settings::load();
    let existed = app.get_webview_window(floatbar_label()).is_some();
    show_bar_window_inner(
        app,
        settings.float_bar_opacity,
        &settings.float_bar_orientation,
        settings.float_bar_click_through,
    )?;

    if let Some(window) = app.get_webview_window(floatbar_label()) {
        if !existed {
            let pos = window.outer_position().map_err(|err| err.to_string())?;
            runtime.emit(LifecycleEvent::Created {
                monitor: None,
                pos: [pos.x, pos.y],
            });
        }
        runtime.emit(LifecycleEvent::Shown);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_transitions_to_hidden_on_hide() {
        assert_eq!(BarState::Visible.transition(BarCommand::Hide), BarState::IntentionallyHidden);
    }

    #[test]
    fn repeated_recovery_failures_pause_the_bar() {
        assert_eq!(
            BarState::Recovering.transition(BarCommand::RecoveryFailed {
                consecutive_failures: 5,
            }),
            BarState::Paused
        );
    }

    #[test]
    fn paused_state_only_restarts_on_retry() {
        assert_eq!(BarState::Paused.transition(BarCommand::Retry), BarState::Visible);
        assert_eq!(BarState::Paused.transition(BarCommand::Show), BarState::Paused);
    }

    #[test]
    fn watchdog_is_only_active_when_visible() {
        assert!(BarState::Visible.watchdog_active());
        assert!(!BarState::IntentionallyHidden.watchdog_active());
        assert!(!BarState::Recovering.watchdog_active());
        assert!(!BarState::Paused.watchdog_active());
    }
}
