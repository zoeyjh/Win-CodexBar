use std::path::PathBuf;

pub mod backoff;
pub mod lifecycle_event;
pub mod logger;
pub mod state;
pub mod watchdog;
pub mod window_state;

const APP_DIR_NAME: &str = ".codexbar-zoey";

pub fn data_root_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR_NAME)
}

pub fn lifecycle_log_dir() -> PathBuf {
    data_root_dir().join("logs")
}

pub fn window_state_path() -> PathBuf {
    data_root_dir().join("window.json")
}
