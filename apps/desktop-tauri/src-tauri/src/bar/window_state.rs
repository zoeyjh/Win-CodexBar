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
                w: 300,
                h: 40,
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
}
