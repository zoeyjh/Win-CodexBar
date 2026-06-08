use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub rect: WindowRect,
    pub monitor_id: Option<String>,
    pub dpi: Option<f64>,
    pub saved_at: DateTime<Utc>,
    pub coord_space: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl WindowState {
    pub fn default_position() -> Self {
        Self {
            rect: WindowRect {
                x: 1600,
                y: 8,
                w: 380,
                h: 220,
            },
            monitor_id: None,
            dpi: None,
            saved_at: Utc::now(),
            coord_space: "logical".to_string(),
        }
    }

    pub fn load(path: &PathBuf) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_else(|_| Self::default_position())
            }
            Err(_) => Self::default_position(),
        }
    }

    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    pub fn clamp_to_work_area(&mut self, work_area: (i32, i32, u32, u32)) {
        let (wa_x, wa_y, wa_w, wa_h) = work_area;
        let max_x = wa_x + wa_w as i32 - self.rect.w as i32;
        let max_y = wa_y + wa_h as i32 - self.rect.h as i32;
        self.rect.x = self.rect.x.clamp(wa_x, max_x.max(wa_x));
        self.rect.y = self.rect.y.clamp(wa_y, max_y.max(wa_y));
    }

    /// Pick the monitor work area to clamp the restored window against.
    ///
    /// Monitors are `(name, x, y, w, h)`. Preference order:
    /// 1. the monitor the bar was last shown on (matched by `monitor_id`),
    /// 2. the monitor whose bounds contain the saved top-left corner,
    /// 3. the first monitor.
    ///
    /// Returns `None` when there are no monitors. Without this, a bar saved on
    /// a secondary monitor is always clamped to the first/primary monitor on
    /// restore, yanking it to the wrong screen.
    pub fn choose_work_area(
        &self,
        monitors: &[(Option<String>, i32, i32, u32, u32)],
    ) -> Option<(i32, i32, u32, u32)> {
        if let Some(id) = self.monitor_id.as_deref()
            && let Some(m) = monitors
                .iter()
                .find(|(name, ..)| name.as_deref() == Some(id))
        {
            return Some((m.1, m.2, m.3, m.4));
        }

        if let Some(m) = monitors.iter().find(|&&(_, mx, my, mw, mh)| {
            self.rect.x >= mx
                && self.rect.y >= my
                && self.rect.x < mx + mw as i32
                && self.rect.y < my + mh as i32
        }) {
            return Some((m.1, m.2, m.3, m.4));
        }

        monitors.first().map(|m| (m.1, m.2, m.3, m.4))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dual_monitors() -> Vec<(Option<String>, i32, i32, u32, u32)> {
        vec![
            (Some(r"\\.\DISPLAY1".to_string()), 0, 0, 1920, 1080),
            (Some(r"\\.\DISPLAY2".to_string()), 1920, 0, 1920, 1080),
        ]
    }

    fn state_at(x: i32, y: i32, monitor_id: Option<&str>) -> WindowState {
        WindowState {
            rect: WindowRect { x, y, w: 380, h: 220 },
            monitor_id: monitor_id.map(str::to_string),
            dpi: Some(1.0),
            saved_at: Utc::now(),
            coord_space: "logical".to_string(),
        }
    }

    #[test]
    fn prefers_the_monitor_the_bar_was_saved_on() {
        // Saved on DISPLAY2 → must clamp against DISPLAY2, not the primary.
        let state = state_at(3444, 947, Some(r"\\.\DISPLAY2"));
        assert_eq!(
            state.choose_work_area(&dual_monitors()),
            Some((1920, 0, 1920, 1080))
        );
    }

    #[test]
    fn falls_back_to_monitor_containing_the_point_when_id_is_unknown() {
        // No monitor id (or an unplugged one) → use the monitor under the
        // saved top-left corner. (3444,947) is inside DISPLAY2.
        let state = state_at(3444, 947, None);
        assert_eq!(
            state.choose_work_area(&dual_monitors()),
            Some((1920, 0, 1920, 1080))
        );
    }

    #[test]
    fn falls_back_to_first_monitor_when_nothing_matches() {
        // Stranded point on no monitor, unknown id → first monitor.
        let state = state_at(5000, 2000, Some(r"\\.\DISPLAY9"));
        assert_eq!(
            state.choose_work_area(&dual_monitors()),
            Some((0, 0, 1920, 1080))
        );
    }

    #[test]
    fn repro_secondary_monitor_bar_stays_on_secondary_after_clamp() {
        // Full repro: a bar dropped low on DISPLAY2 used to be clamped to
        // DISPLAY1 (the [1540,860] bug). It must now stay on DISPLAY2.
        let mut state = state_at(3444, 947, Some(r"\\.\DISPLAY2"));
        let area = state.choose_work_area(&dual_monitors()).unwrap();
        state.clamp_to_work_area(area);
        assert!(state.rect.x >= 1920, "bar must remain on DISPLAY2");
        assert!(state.rect.x + state.rect.w as i32 <= 3840);
        assert!(state.rect.y + state.rect.h as i32 <= 1080);
    }

    #[test]
    fn no_monitors_returns_none() {
        let state = state_at(100, 100, None);
        assert_eq!(state.choose_work_area(&[]), None);
    }
}
