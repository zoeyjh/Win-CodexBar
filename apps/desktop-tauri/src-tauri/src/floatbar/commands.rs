//! Tauri commands that drive the floating-bar window.
//!
//! These keep persisted settings in sync, then forward visibility changes
//! through the shared bar actor so watchdog state and lifecycle logging stay
//! coherent.

use codexbar::settings::{Settings, clamp_float_bar_opacity, normalize_float_bar_orientation};
use tauri::{Emitter, Manager, State};

use crate::bar::state::{BarCommand, BarRuntimeState};

use super::window as floatbar_window;

#[tauri::command]
pub async fn show_float_bar(bar_runtime: State<'_, BarRuntimeState>) -> Result<(), String> {
    let runtime = bar_runtime.inner().clone();
    let mut settings = Settings::load();
    settings.float_bar_enabled = true;
    settings.save().map_err(|e| e.to_string())?;
    runtime.send(BarCommand::Show).await
}

#[tauri::command]
pub fn hide_float_bar(bar_runtime: State<'_, BarRuntimeState>) -> Result<(), String> {
    let runtime = bar_runtime.inner().clone();
    let mut settings = Settings::load();
    settings.float_bar_enabled = false;
    settings.save().map_err(|e| e.to_string())?;
    runtime.try_send(BarCommand::Hide)
}

#[tauri::command]
pub fn set_float_bar_opacity(app: tauri::AppHandle, opacity: u8) -> Result<(), String> {
    let opacity = clamp_float_bar_opacity(opacity);
    let mut settings = Settings::load();
    settings.float_bar_opacity = opacity;
    settings.save().map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window(floatbar_window::FLOATBAR_LABEL) {
        floatbar_window::apply_opacity(&window, opacity);
    }
    Ok(())
}

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

#[tauri::command]
pub fn set_float_bar_orientation(app: tauri::AppHandle, orientation: String) -> Result<(), String> {
    let orientation = normalize_float_bar_orientation(&orientation);
    let mut settings = Settings::load();
    settings.float_bar_orientation = orientation;
    settings.save().map_err(|e| e.to_string())?;

    let _ = app.emit(super::FLOAT_BAR_CONFIG_CHANGED_EVENT, ());
    Ok(())
}

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
