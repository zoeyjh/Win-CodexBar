# Floating Token Bar (codexbar-zoey) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimal, always-on-top floating bar showing Claude/Codex/Copilot token usage with auto-recovery watchdog, forked from Win-CodexBar.

**Architecture:** Fork Win-CodexBar → strip 46 unused providers → rewrite FloatBar UI (React) to ultra-minimal + rewrite Tauri window lifecycle (Rust) with state machine, watchdog, and lifecycle logger. Keep data layer (provider auth/polling) intact. Detail view with 3 cards for deeper usage info.

**Tech Stack:** Tauri v1 (or v2 per upstream), React, TypeScript, Rust, tokio, serde. Chart lib TBD (R-5). Design tokens from `docs/design/design-notes.md`.

---

## Phase 0: Setup & Pre-Implementation Research

> This phase must complete before any code changes. Results may alter later tasks.

### Task 0.1: Clone Win-CodexBar and set up project

**Files:**
- Create: `E:\04_Dev\02_tool\codexbar-zoey\` (entire repo via clone)

- [ ] **Step 1: Clone the upstream repo into codexbar-zoey**

```powershell
cd E:\04_Dev\02_tool
git clone https://github.com/Finesssee/Win-CodexBar.git codexbar-zoey
cd codexbar-zoey
```

- [ ] **Step 2: Rename origin to upstream, set up branches**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey
git remote rename origin upstream
git checkout -b zoey-mvp
```

- [ ] **Step 3: Verify docs/ folder is preserved (our specs/design)**

Our `docs/` folder already exists with specs and design. If the clone overwrote it, restore from backup. The clone should have merged since docs/superpowers and docs/design don't exist in upstream.

```powershell
# If docs/ was lost, copy back from a backup or re-create
# Verify:
Test-Path "E:\04_Dev\02_tool\codexbar-zoey\docs\superpowers\specs\2026-05-28-floating-token-bar-design.md"
Test-Path "E:\04_Dev\02_tool\codexbar-zoey\docs\design\design-notes.md"
```

> **NOTE:** If upstream has a `docs/` folder that conflicts, move our files to a non-conflicting path or use a pre-clone strategy: clone to a temp dir, then copy contents into the existing `codexbar-zoey` directory that already has `docs/`.

Alternative approach (safer):
```powershell
cd E:\04_Dev\02_tool
# Back up existing docs
Copy-Item -Recurse "codexbar-zoey\docs" "codexbar-zoey-docs-backup"
# Clone into temp
git clone https://github.com/Finesssee/Win-CodexBar.git codexbar-zoey-temp
# Copy upstream contents into our folder
Copy-Item -Recurse -Force "codexbar-zoey-temp\*" "codexbar-zoey\"
# Restore docs
Copy-Item -Recurse -Force "codexbar-zoey-docs-backup\*" "codexbar-zoey\docs\"
# Clean temp
Remove-Item -Recurse -Force "codexbar-zoey-temp"
Remove-Item -Recurse -Force "codexbar-zoey-docs-backup"
# Init git properly
cd codexbar-zoey
git init
git remote add upstream https://github.com/Finesssee/Win-CodexBar.git
git add -A
git commit -m "chore: initial fork of Win-CodexBar + project docs"
git checkout -b zoey-mvp
```

- [ ] **Step 4: Add .gitignore entries**

Append to `.gitignore` (create if not exists):
```
.superpowers/
node_modules/
target/
dist/
```

- [ ] **Step 5: Commit setup**

```powershell
git add .gitignore
git commit -m "chore: add .gitignore for superpowers, node_modules, target"
```

---

### Task 0.2: Research R-1 — Provider data fields

**Goal:** Determine what fields each provider API actually returns (used/limit/reset_at/sessions).

**Files:**
- Read: `rust/src/providers/claude/` (or wherever Claude provider lives)
- Read: `rust/src/providers/codex/` (or openai/)
- Read: `rust/src/providers/copilot/` (or github/)
- Create: `docs/research/R1-provider-fields.md`

- [ ] **Step 1: Find and read Claude provider code**

```powershell
# Find all provider directories
Get-ChildItem "E:\04_Dev\02_tool\codexbar-zoey\rust\src\providers" -Directory | Select-Object Name
# Then read the main file of each relevant provider
```

- [ ] **Step 2: Find and read Codex/OpenAI provider code**

Look for response parsing — what fields does the API return? Document: `used`, `limit`, `reset_at`, `plan`, `model_breakdown`, `sessions`.

- [ ] **Step 3: Find and read Copilot/GitHub provider code**

Same analysis.

- [ ] **Step 4: Write findings**

Create `docs/research/R1-provider-fields.md`:
```markdown
# R-1: Provider Data Fields

| Field | Claude | Codex (OpenAI) | Copilot (GitHub) |
|---|---|---|---|
| used | ? | ? | ? |
| limit | ? | ? | ? |
| remaining_pct | ? | ? | ? |
| reset_at | ? | ? | ? |
| plan | ? | ? | ? |
| sessions | ? | ? | ? |
| model_breakdown | ? | ? | ? |

## Notes
- (findings per provider)
```

- [ ] **Step 5: Commit research**

```powershell
git add docs/research/R1-provider-fields.md
git commit -m "docs: R-1 provider data field research"
```

---

### Task 0.3: Research R-2 — Session log sources

**Goal:** Determine if CLI tools leave local session logs we can read for Card 2.

**Files:**
- Create: `docs/research/R2-session-logs.md`

- [ ] **Step 1: Check local filesystem for CLI log files**

```powershell
# Claude CLI logs
Get-ChildItem "$env:USERPROFILE\.claude" -Recurse -Filter "*.jsonl" -ErrorAction SilentlyContinue | Select-Object FullName, Length
# Codex CLI logs
Get-ChildItem "$env:USERPROFILE\.codex" -Recurse -Filter "*.jsonl" -ErrorAction SilentlyContinue | Select-Object FullName, Length
# Copilot CLI logs (if any)
Get-ChildItem "$env:USERPROFILE\.copilot" -Recurse -ErrorAction SilentlyContinue | Select-Object FullName, Length
```

- [ ] **Step 2: Examine log file format (if found)**

Read first few lines of any found JSONL files to understand schema.

- [ ] **Step 3: Document findings**

Create `docs/research/R2-session-logs.md` with findings per CLI.

- [ ] **Step 4: Commit**

```powershell
git add docs/research/R2-session-logs.md
git commit -m "docs: R-2 session log source research"
```

---

### Task 0.4: Research R-3 — Provider dependency graph

**Goal:** Map all files that reference the 46 providers to be removed, beyond just their directories.

**Files:**
- Create: `docs/research/R3-provider-deps.md`

- [ ] **Step 1: List all provider directories**

```powershell
Get-ChildItem "E:\04_Dev\02_tool\codexbar-zoey\rust\src\providers" -Directory | Select-Object Name
```

- [ ] **Step 2: Search for provider registry/enum that lists all providers**

```powershell
# Look for enum or registry that lists all provider names
Select-String -Path "E:\04_Dev\02_tool\codexbar-zoey\rust\src\**\*.rs" -Pattern "enum.*Provider|provider.*registry|PROVIDERS" -Recurse | Select-Object Path, LineNumber, Line
```

- [ ] **Step 3: Search React side for provider references**

```powershell
Select-String -Path "E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri\src\**\*.ts*" -Pattern "provider" -Recurse | Select-Object Path -Unique
```

- [ ] **Step 4: Search Cargo.toml for feature flags related to providers**

```powershell
Get-Content "E:\04_Dev\02_tool\codexbar-zoey\rust\Cargo.toml" | Select-String "provider|claude|codex|copilot|openai|github"
```

- [ ] **Step 5: Document the full dependency map**

Create `docs/research/R3-provider-deps.md`:
- List of files to modify (registry enums, UI mappings, aggregators)
- List of directories to delete
- Estimated effort

- [ ] **Step 6: Commit**

```powershell
git add docs/research/R3-provider-deps.md
git commit -m "docs: R-3 provider dependency graph research"
```

---

### Task 0.5: Research R-4 — Tauri window API capabilities

**Goal:** Verify which Tauri APIs are available for watchdog health checks.

**Files:**
- Create: `docs/research/R4-tauri-window-api.md`

- [ ] **Step 1: Check Tauri version in use**

```powershell
Get-Content "E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri\src-tauri\Cargo.toml" | Select-String "tauri"
```

- [ ] **Step 2: Search codebase for existing window API usage**

```powershell
Select-String -Path "E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri\src-tauri\src\**\*.rs" -Pattern "is_visible|is_always_on_top|current_monitor|outer_position|inner_size" -Recurse
```

- [ ] **Step 3: Check Tauri docs for available APIs**

Document availability of:
- `window.is_visible()`
- `window.is_always_on_top()` (may not exist in Tauri v1)
- `window.current_monitor()`
- `monitor.position()` / `monitor.size()` for work area
- Any occlusion detection

- [ ] **Step 4: Document findings and fallback strategies**

Create `docs/research/R4-tauri-window-api.md`.

- [ ] **Step 5: Commit**

```powershell
git add docs/research/R4-tauri-window-api.md
git commit -m "docs: R-4 Tauri window API research"
```

---

### Task 0.6: Research R-5 — Chart library

**Goal:** Identify chart library already in use or pick one.

**Files:**
- Create: `docs/research/R5-chart-library.md`

- [ ] **Step 1: Check package.json for existing chart deps**

```powershell
Get-Content "E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri\package.json" | Select-String "chart|recharts|d3|victory|nivo|visx"
```

- [ ] **Step 2: Decision — use existing or pick lightweight option**

If nothing found, recommend `recharts` (React-native, lightweight, good for time series).

- [ ] **Step 3: Document**

Create `docs/research/R5-chart-library.md`.

- [ ] **Step 4: Commit**

```powershell
git add docs/research/R5-chart-library.md
git commit -m "docs: R-5 chart library research"
```

---

## Phase 1: Provider Cleanup (Strip 46 providers)

> Depends on R-3 results. Tasks below are templates — exact file paths come from R-3.

### Task 1.1: Remove unused provider directories

**Files:**
- Delete: `rust/src/providers/{all except claude, codex/openai, copilot/github}`

- [ ] **Step 1: Delete provider directories**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey
# Keep only the 3 needed providers (exact names from R-3)
# Example (names will vary):
$keep = @("claude", "openai", "github")  # or "codex", "copilot" — use actual names from R-3
$all = Get-ChildItem "rust\src\providers" -Directory | Where-Object { $_.Name -notin $keep }
$all | Remove-Item -Recurse -Force
```

- [ ] **Step 2: Update Rust module declarations**

Edit `rust/src/providers/mod.rs` (or equivalent) to only declare the 3 kept modules. Remove all `pub mod <deleted_provider>;` lines.

- [ ] **Step 3: Update provider registry/enum**

Edit the enum/registry file found in R-3. Keep only 3 variants.

- [ ] **Step 4: Fix Cargo.toml features**

Remove feature flags for deleted providers.

- [ ] **Step 5: Attempt build to find remaining references**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo build 2>&1 | Select-String "error"
```

- [ ] **Step 6: Fix all remaining compile errors**

Iteratively fix errors until `cargo build` succeeds.

- [ ] **Step 7: Commit**

```powershell
git add -A
git commit -m "feat: strip 46 unused providers, keep claude/codex/copilot only"
```

---

### Task 1.2: Remove provider references from React UI

**Files:**
- Modify: Provider list components, settings pages (paths from R-3)

- [ ] **Step 1: Find UI files referencing removed providers**

```powershell
Select-String -Path "E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri\src\**\*" -Pattern "provider" -Recurse | Select-Object Path -Unique
```

- [ ] **Step 2: Update provider lists/configs in React to only show 3**

- [ ] **Step 3: Verify app still builds**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri
npm install
npm run build
```

- [ ] **Step 4: Commit**

```powershell
git add -A
git commit -m "feat: strip unused provider UI references"
```

---

## Phase 2: Rust Core — State Machine, Watchdog, Logger

### Task 2.1: Define BarState state machine

**Files:**
- Create: `rust/src/tauri_app/bar_state.rs`
- Modify: `rust/src/tauri_app/mod.rs` (add module declaration)

- [ ] **Step 1: Write tests for state transitions**

Create `rust/src/tauri_app/bar_state.rs`:

```rust
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
        use BarState::*;
        use BarCommand::*;
        match (self, cmd) {
            (Visible, Hide) => IntentionallyHidden,
            (Visible, StartRecovery) => Recovering,
            (Visible, Quit) => Quitting,

            (IntentionallyHidden, Show) => Visible,
            (IntentionallyHidden, Quit) => Quitting,

            (Recovering, RecoverySuccess) => Visible,
            (Recovering, RecoveryFailed { consecutive_failures }) if consecutive_failures >= 5 => Paused,
            (Recovering, RecoveryFailed { .. }) => Recovering,
            (Recovering, Quit) => Quitting,

            (Paused, Retry) => Visible,
            (Paused, Quit) => Quitting,

            // No-op transitions (stay in same state)
            _ => self,
        }
    }

    pub fn watchdog_active(self) -> bool {
        matches!(self, BarState::Visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_hide_becomes_intentionally_hidden() {
        let state = BarState::Visible.transition(BarCommand::Hide);
        assert_eq!(state, BarState::IntentionallyHidden);
    }

    #[test]
    fn intentionally_hidden_show_becomes_visible() {
        let state = BarState::IntentionallyHidden.transition(BarCommand::Show);
        assert_eq!(state, BarState::Visible);
    }

    #[test]
    fn visible_start_recovery_becomes_recovering() {
        let state = BarState::Visible.transition(BarCommand::StartRecovery);
        assert_eq!(state, BarState::Recovering);
    }

    #[test]
    fn recovering_success_becomes_visible() {
        let state = BarState::Recovering.transition(BarCommand::RecoverySuccess);
        assert_eq!(state, BarState::Visible);
    }

    #[test]
    fn recovering_five_failures_becomes_paused() {
        let state = BarState::Recovering.transition(BarCommand::RecoveryFailed { consecutive_failures: 5 });
        assert_eq!(state, BarState::Paused);
    }

    #[test]
    fn recovering_fewer_failures_stays_recovering() {
        let state = BarState::Recovering.transition(BarCommand::RecoveryFailed { consecutive_failures: 3 });
        assert_eq!(state, BarState::Recovering);
    }

    #[test]
    fn paused_retry_becomes_visible() {
        let state = BarState::Paused.transition(BarCommand::Retry);
        assert_eq!(state, BarState::Visible);
    }

    #[test]
    fn any_state_quit_becomes_quitting() {
        for state in [BarState::Visible, BarState::IntentionallyHidden, BarState::Recovering, BarState::Paused] {
            assert_eq!(state.transition(BarCommand::Quit), BarState::Quitting);
        }
    }

    #[test]
    fn watchdog_only_active_when_visible() {
        assert!(BarState::Visible.watchdog_active());
        assert!(!BarState::IntentionallyHidden.watchdog_active());
        assert!(!BarState::Recovering.watchdog_active());
        assert!(!BarState::Paused.watchdog_active());
        assert!(!BarState::Quitting.watchdog_active());
    }
}
```

- [ ] **Step 2: Run tests**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo test bar_state -- --nocapture
```

Expected: All tests pass.

- [ ] **Step 3: Add module to mod.rs**

Add `pub mod bar_state;` to `rust/src/tauri_app/mod.rs`.

- [ ] **Step 4: Commit**

```powershell
git add rust/src/tauri_app/bar_state.rs rust/src/tauri_app/mod.rs
git commit -m "feat: add BarState state machine with full test coverage"
```

---

### Task 2.2: Define LifecycleEvent types

**Files:**
- Create: `rust/src/tauri_app/lifecycle_event.rs`
- Modify: `rust/src/tauri_app/mod.rs`

- [ ] **Step 1: Create lifecycle event enum**

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::bar_state::BarState;

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
```

- [ ] **Step 2: Add module declaration**

Add `pub mod lifecycle_event;` to `rust/src/tauri_app/mod.rs`.

- [ ] **Step 3: Verify it compiles**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo check
```

- [ ] **Step 4: Commit**

```powershell
git add rust/src/tauri_app/lifecycle_event.rs rust/src/tauri_app/mod.rs
git commit -m "feat: define LifecycleEvent types for bar lifecycle tracking"
```

---

### Task 2.3: Implement LifecycleLogger

**Files:**
- Create: `rust/src/tauri_app/lifecycle_logger.rs`
- Modify: `rust/src/tauri_app/mod.rs`

- [ ] **Step 1: Write the logger with redaction and rotation**

```rust
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use chrono::{Local, Utc, Duration};
use tokio::sync::broadcast;

use super::lifecycle_event::TimestampedEvent;

pub struct LifecycleLogger {
    log_dir: PathBuf,
    retention_days: i64,
}

impl LifecycleLogger {
    pub fn new(data_dir: &PathBuf) -> Self {
        let log_dir = data_dir.join("logs");
        fs::create_dir_all(&log_dir).ok();
        Self {
            log_dir,
            retention_days: 7,
        }
    }

    pub fn log_file_path(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d");
        self.log_dir.join(format!("lifecycle-{}.log", date))
    }

    pub fn append(&self, event: &TimestampedEvent) -> std::io::Result<()> {
        let path = self.log_file_path();
        let line = serde_json::to_string(event)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    pub fn rotate(&self) {
        let cutoff = Utc::now() - Duration::days(self.retention_days);
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Parse date from "lifecycle-YYYY-MM-DD.log"
                    if let Some(date_str) = name.strip_prefix("lifecycle-").and_then(|s| s.strip_suffix(".log")) {
                        if let Ok(file_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                            let file_dt = file_date.and_hms_opt(0, 0, 0).unwrap();
                            let file_utc = DateTime::<Utc>::from_naive_utc_and_offset(file_dt, Utc);
                            if file_utc < cutoff {
                                fs::remove_file(&path).ok();
                            }
                        }
                    }
                }
            }
        }
    }

    /// Spawn a background task that listens to lifecycle events and logs them.
    /// `watchdog_tick` events are throttled to 1 "ok" per 5 minutes.
    pub async fn run(self, mut rx: broadcast::Receiver<TimestampedEvent>) {
        use super::lifecycle_event::LifecycleEvent;
        use std::time::Instant;

        let mut last_ok_tick = Instant::now() - std::time::Duration::from_secs(300);

        // Rotate on startup
        self.rotate();

        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Throttle watchdog_tick OK events
                    if let LifecycleEvent::WatchdogTick { healthy: true } = &event.event {
                        if last_ok_tick.elapsed() < std::time::Duration::from_secs(300) {
                            continue;
                        }
                        last_ok_tick = Instant::now();
                    }

                    if let Err(e) = self.append(&event) {
                        eprintln!("LifecycleLogger write error: {}", e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("LifecycleLogger lagged by {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lifecycle_event::LifecycleEvent;
    use tempfile::TempDir;

    #[test]
    fn append_creates_log_file_and_writes_json_line() {
        let tmp = TempDir::new().unwrap();
        let logger = LifecycleLogger::new(&tmp.path().to_path_buf());

        let event = TimestampedEvent::now(LifecycleEvent::Created {
            monitor: Some(0),
            pos: [100, 50],
        });

        logger.append(&event).unwrap();

        let path = logger.log_file_path();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"event\":\"created\""));
        assert!(content.contains("\"monitor\":0"));
    }

    #[test]
    fn rotate_removes_old_files() {
        let tmp = TempDir::new().unwrap();
        let logger = LifecycleLogger::new(&tmp.path().to_path_buf());

        // Create a fake old log file (10 days ago)
        let old_date = (Local::now() - Duration::days(10)).format("%Y-%m-%d");
        let old_file = logger.log_dir.join(format!("lifecycle-{}.log", old_date));
        fs::write(&old_file, "old entry\n").unwrap();

        // Create a recent log file (1 day ago)
        let recent_date = (Local::now() - Duration::days(1)).format("%Y-%m-%d");
        let recent_file = logger.log_dir.join(format!("lifecycle-{}.log", recent_date));
        fs::write(&recent_file, "recent entry\n").unwrap();

        logger.rotate();

        assert!(!old_file.exists(), "Old file should be deleted");
        assert!(recent_file.exists(), "Recent file should be kept");
    }
}
```

- [ ] **Step 2: Run tests**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo test lifecycle_logger -- --nocapture
```

- [ ] **Step 3: Add module declaration and tempfile dev-dep**

In `Cargo.toml`, add `tempfile = "3"` under `[dev-dependencies]`.
Add `pub mod lifecycle_logger;` to `rust/src/tauri_app/mod.rs`.

- [ ] **Step 4: Commit**

```powershell
git add -A
git commit -m "feat: implement LifecycleLogger with rotation and throttling"
```

---

### Task 2.4: Implement BarWatchdog

**Files:**
- Create: `rust/src/tauri_app/bar_watchdog.rs`
- Modify: `rust/src/tauri_app/mod.rs`

- [ ] **Step 1: Define WindowHealthChecker trait (for testability)**

```rust
use async_trait::async_trait;

/// Abstraction over Tauri window API for testability
#[async_trait]
pub trait WindowHealthChecker: Send + Sync {
    fn is_handle_valid(&self) -> bool;
    fn is_visible(&self) -> bool;
    /// Returns None if API not available (fallback to last setter time)
    fn is_always_on_top(&self) -> Option<bool>;
    /// Returns (x, y, w, h) of the window in logical coords
    fn outer_rect(&self) -> Option<(i32, i32, u32, u32)>;
    /// Returns available work areas: Vec<(x, y, w, h)>
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
```

- [ ] **Step 2: Write watchdog tick logic with tests**

```rust
pub struct WatchdogState {
    consecutive_failures: u32,
    consecutive_recovery_failures: u32,
}

impl WatchdogState {
    pub fn new() -> Self {
        Self {
            consecutive_failures: 0,
            consecutive_recovery_failures: 0,
        }
    }

    /// Returns the command to send to BarState actor, if any
    pub fn tick(&mut self, checker: &dyn WindowHealthChecker) -> Option<super::bar_state::BarCommand> {
        let healthy = self.check_health(checker);
        if healthy {
            self.consecutive_failures = 0;
            None
        } else {
            self.consecutive_failures += 1;
            if self.consecutive_failures >= 3 {
                self.consecutive_failures = 0;
                self.consecutive_recovery_failures += 1;
                Some(super::bar_state::BarCommand::StartRecovery)
            } else {
                None
            }
        }
    }

    pub fn on_recovery_success(&mut self) {
        self.consecutive_recovery_failures = 0;
    }

    pub fn on_recovery_failed(&mut self) -> super::bar_state::BarCommand {
        super::bar_state::BarCommand::RecoveryFailed {
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
        // is_always_on_top: if API not available, assume OK (best-effort)
        if let Some(on_top) = checker.is_always_on_top() {
            if !on_top {
                return false;
            }
        }
        // Check if within any work area
        if let Some(rect) = checker.outer_rect() {
            let areas = checker.available_work_areas();
            if !areas.is_empty() && !is_within_work_areas(rect, &areas) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::bar_state::BarCommand;

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
                rect: Some((100, 50, 300, 40)),
                work_areas: vec![(0, 0, 1920, 1040)],
            }
        }

        fn unhealthy_invisible() -> Self {
            Self { visible: false, ..Self::healthy() }
        }

        fn unhealthy_off_screen() -> Self {
            Self {
                rect: Some((-500, -500, 300, 40)),
                ..Self::healthy()
            }
        }
    }

    impl WindowHealthChecker for MockChecker {
        fn is_handle_valid(&self) -> bool { self.handle_valid }
        fn is_visible(&self) -> bool { self.visible }
        fn is_always_on_top(&self) -> Option<bool> { self.always_on_top }
        fn outer_rect(&self) -> Option<(i32, i32, u32, u32)> { self.rect }
        fn available_work_areas(&self) -> Vec<(i32, i32, u32, u32)> { self.work_areas.clone() }
    }

    #[test]
    fn healthy_window_resets_failure_count() {
        let mut wd = WatchdogState::new();
        wd.consecutive_failures = 2;
        let cmd = wd.tick(&MockChecker::healthy());
        assert_eq!(cmd, None);
        assert_eq!(wd.consecutive_failures, 0);
    }

    #[test]
    fn three_consecutive_failures_triggers_recovery() {
        let mut wd = WatchdogState::new();
        let checker = MockChecker::unhealthy_invisible();
        assert_eq!(wd.tick(&checker), None); // 1
        assert_eq!(wd.tick(&checker), None); // 2
        assert!(matches!(wd.tick(&checker), Some(BarCommand::StartRecovery))); // 3
    }

    #[test]
    fn off_screen_is_unhealthy() {
        let mut wd = WatchdogState::new();
        let checker = MockChecker::unhealthy_off_screen();
        wd.tick(&checker);
        wd.tick(&checker);
        let cmd = wd.tick(&checker);
        assert!(matches!(cmd, Some(BarCommand::StartRecovery)));
    }

    #[test]
    fn work_area_check_passes_when_inside() {
        let pos = (100, 50, 300, 40);
        let areas = vec![(0, 0, 1920, 1080)];
        assert!(is_within_work_areas(pos, &areas));
    }

    #[test]
    fn work_area_check_fails_when_outside() {
        let pos = (-100, 50, 300, 40);
        let areas = vec![(0, 0, 1920, 1080)];
        assert!(!is_within_work_areas(pos, &areas));
    }
}
```

- [ ] **Step 3: Run tests**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo test bar_watchdog -- --nocapture
```

- [ ] **Step 4: Add module declaration**

Add `pub mod bar_watchdog;` to `rust/src/tauri_app/mod.rs`.

- [ ] **Step 5: Commit**

```powershell
git add -A
git commit -m "feat: implement BarWatchdog with trait-based health checking"
```

---

### Task 2.5: Implement UsageSnapshot extended schema

**Files:**
- Create: `rust/src/tauri_app/usage_snapshot.rs` (or modify existing status types)
- Modify: `rust/src/tauri_app/mod.rs`

- [ ] **Step 1: Define the extended UsageSnapshot types**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    Claude,
    Codex,
    Copilot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Token,
    Request,
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotStatus {
    Ok,
    Stale,
    AuthExpired,
    RateLimited,
    Network,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Cached,
    Inferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub provider: ProviderId,
    pub plan: Option<String>,
    pub unit: Unit,
    pub used: Option<u64>,
    pub limit: Option<u64>,
    pub remaining_pct: Option<f32>,
    pub reset_at: Option<DateTime<Utc>>,
    pub status: SnapshotStatus,
    pub last_success_at: DateTime<Utc>,
    pub confidence: Confidence,
}

impl UsageSnapshot {
    /// Compute remaining_pct from used/limit if not directly provided
    pub fn compute_remaining_pct(&mut self) {
        if self.remaining_pct.is_none() {
            if let (Some(used), Some(limit)) = (self.used, self.limit) {
                if limit > 0 {
                    let remaining = limit.saturating_sub(used) as f32 / limit as f32 * 100.0;
                    self.remaining_pct = Some(remaining.clamp(0.0, 100.0));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_remaining_pct_from_used_limit() {
        let mut snap = UsageSnapshot {
            provider: ProviderId::Claude,
            plan: Some("Pro".to_string()),
            unit: Unit::Token,
            used: Some(300),
            limit: Some(1000),
            remaining_pct: None,
            reset_at: None,
            status: SnapshotStatus::Ok,
            last_success_at: Utc::now(),
            confidence: Confidence::High,
        };
        snap.compute_remaining_pct();
        assert_eq!(snap.remaining_pct, Some(70.0));
    }

    #[test]
    fn compute_remaining_pct_does_not_override_existing() {
        let mut snap = UsageSnapshot {
            provider: ProviderId::Copilot,
            plan: None,
            unit: Unit::Request,
            used: Some(50),
            limit: Some(100),
            remaining_pct: Some(45.0), // already set (from API)
            reset_at: None,
            status: SnapshotStatus::Ok,
            last_success_at: Utc::now(),
            confidence: Confidence::High,
        };
        snap.compute_remaining_pct();
        assert_eq!(snap.remaining_pct, Some(45.0)); // not overridden
    }
}
```

- [ ] **Step 2: Run tests**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo test usage_snapshot -- --nocapture
```

- [ ] **Step 3: Add module and commit**

```powershell
git add -A
git commit -m "feat: define extended UsageSnapshot schema"
```

---

### Task 2.6: Implement Backoff policy

**Files:**
- Create: `rust/src/tauri_app/backoff.rs`

- [ ] **Step 1: Write backoff calculator with tests**

```rust
use std::time::Duration;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct BackoffPolicy {
    base: Duration,
    max: Duration,
    factor: f64,
    jitter_pct: f64,
    current_attempt: u32,
}

impl BackoffPolicy {
    pub fn new(base: Duration, max: Duration, factor: f64, jitter_pct: f64) -> Self {
        Self { base, max, factor, jitter_pct, current_attempt: 0 }
    }

    pub fn next_delay(&mut self) -> Duration {
        let delay_secs = self.base.as_secs_f64() * self.factor.powi(self.current_attempt as i32);
        let capped = delay_secs.min(self.max.as_secs_f64());

        // Apply jitter
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-self.jitter_pct..=self.jitter_pct);
        let final_secs = capped * (1.0 + jitter);

        self.current_attempt += 1;
        Duration::from_secs_f64(final_secs.max(0.0))
    }

    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }
}

/// Default policies per spec
pub fn rate_limit_backoff() -> BackoffPolicy {
    BackoffPolicy::new(
        Duration::from_secs(60),
        Duration::from_secs(30 * 60),
        2.0,
        0.2,
    )
}

pub fn server_error_backoff(base_poll_interval: Duration) -> BackoffPolicy {
    BackoffPolicy::new(
        base_poll_interval,
        Duration::from_secs(10 * 60),
        2.0,
        0.2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_delay_is_near_base() {
        let mut bp = BackoffPolicy::new(Duration::from_secs(60), Duration::from_secs(1800), 2.0, 0.0);
        let d = bp.next_delay();
        assert_eq!(d, Duration::from_secs(60));
    }

    #[test]
    fn second_delay_doubles() {
        let mut bp = BackoffPolicy::new(Duration::from_secs(60), Duration::from_secs(1800), 2.0, 0.0);
        bp.next_delay(); // 60
        let d = bp.next_delay(); // 120
        assert_eq!(d, Duration::from_secs(120));
    }

    #[test]
    fn caps_at_max() {
        let mut bp = BackoffPolicy::new(Duration::from_secs(60), Duration::from_secs(300), 2.0, 0.0);
        for _ in 0..10 {
            bp.next_delay();
        }
        let d = bp.next_delay();
        assert!(d <= Duration::from_secs(300));
    }

    #[test]
    fn reset_restarts_from_base() {
        let mut bp = BackoffPolicy::new(Duration::from_secs(60), Duration::from_secs(1800), 2.0, 0.0);
        bp.next_delay(); // 60
        bp.next_delay(); // 120
        bp.reset();
        let d = bp.next_delay(); // should be 60 again
        assert_eq!(d, Duration::from_secs(60));
    }
}
```

- [ ] **Step 2: Run tests and commit**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo test backoff -- --nocapture
git add -A
git commit -m "feat: implement exponential backoff with jitter"
```

---

### Task 2.7: Implement window.json persistence

**Files:**
- Create: `rust/src/tauri_app/window_state.rs`

- [ ] **Step 1: Write WindowState struct with save/load and clamp logic**

```rust
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
    pub coord_space: String, // always "logical"
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
            rect: WindowRect { x: 1600, y: 8, w: 300, h: 40 },
            monitor_id: None,
            dpi: None,
            saved_at: Utc::now(),
            coord_space: "logical".to_string(),
        }
    }

    pub fn load(path: &PathBuf) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::default_position()),
            Err(_) => Self::default_position(),
        }
    }

    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir)?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Clamp the window position to be within the given work area
    pub fn clamp_to_work_area(&mut self, work_area: (i32, i32, u32, u32)) {
        let (wa_x, wa_y, wa_w, wa_h) = work_area;
        let max_x = wa_x + wa_w as i32 - self.rect.w as i32;
        let max_y = wa_y + wa_h as i32 - self.rect.h as i32;

        self.rect.x = self.rect.x.clamp(wa_x, max_x.max(wa_x));
        self.rect.y = self.rect.y.clamp(wa_y, max_y.max(wa_y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("window.json");

        let state = WindowState {
            rect: WindowRect { x: 200, y: 50, w: 320, h: 44 },
            monitor_id: Some("MONITOR-1".to_string()),
            dpi: Some(1.25),
            saved_at: Utc::now(),
            coord_space: "logical".to_string(),
        };

        state.save(&path).unwrap();
        let loaded = WindowState::load(&path);
        assert_eq!(loaded.rect.x, 200);
        assert_eq!(loaded.rect.y, 50);
        assert_eq!(loaded.monitor_id, Some("MONITOR-1".to_string()));
    }

    #[test]
    fn load_missing_file_returns_default() {
        let path = PathBuf::from("/nonexistent/window.json");
        let state = WindowState::load(&path);
        assert_eq!(state.coord_space, "logical");
    }

    #[test]
    fn clamp_negative_coords() {
        let mut state = WindowState::default_position();
        state.rect.x = -100;
        state.rect.y = -50;
        state.clamp_to_work_area((0, 0, 1920, 1040));
        assert_eq!(state.rect.x, 0);
        assert_eq!(state.rect.y, 0);
    }

    #[test]
    fn clamp_beyond_right_edge() {
        let mut state = WindowState::default_position();
        state.rect.x = 2000;
        state.rect.w = 300;
        state.clamp_to_work_area((0, 0, 1920, 1040));
        assert_eq!(state.rect.x, 1620); // 1920 - 300
    }
}
```

- [ ] **Step 2: Run tests and commit**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\rust
cargo test window_state -- --nocapture
git add -A
git commit -m "feat: implement window.json state persistence with clamp"
```

---

## Phase 3: Tauri Window Management (BarWindow rewrite)

### Task 3.1: Rewrite BarWindow creation (runtime WebviewWindowBuilder)

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/main.rs` (or equivalent app entry point)
- Modify: `apps/desktop-tauri/src-tauri/tauri.conf.json` (remove bar window definition)

- [ ] **Step 1: Remove bar window from tauri.conf.json**

Remove any `"windows"` array entry for the float bar. Keep only capabilities/security config.

- [ ] **Step 2: Create bar window at runtime in Rust**

In the app setup hook (or equivalent), use `WebviewWindowBuilder` to create the bar window:

```rust
// In setup or equivalent:
use tauri::WebviewWindowBuilder;

let window_state = WindowState::load(&data_dir.join("window.json"));

let bar_window = WebviewWindowBuilder::new(app, "floatbar", tauri::WebviewUrl::App("floatbar/index.html".into()))
    .title("codexbar-zoey")
    .inner_size(window_state.rect.w as f64, window_state.rect.h as f64)
    .position(window_state.rect.x as f64, window_state.rect.y as f64)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .focused(false)
    .build()?;
```

> **Note:** Exact API depends on Tauri version (v1 vs v2). Adjust method names after R-4 research.

- [ ] **Step 3: Wire up lifecycle event emission on window events**

Subscribe to window events (moved, visibility changed, etc.) and emit `LifecycleEvent` to the broadcast channel.

- [ ] **Step 4: Wire up position saving on window move**

On `tauri::WindowEvent::Moved`, save position to `window.json`.

- [ ] **Step 5: Verify bar window appears on app start**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri
npm run tauri dev
```

- [ ] **Step 6: Commit**

```powershell
git add -A
git commit -m "feat: rewrite bar window to runtime WebviewWindowBuilder"
```

---

### Task 3.2: Integrate Watchdog and Logger into app lifecycle

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/main.rs`

- [ ] **Step 1: Create broadcast channel and spawn logger**

```rust
let (lifecycle_tx, _) = tokio::sync::broadcast::channel::<TimestampedEvent>(256);

// Spawn logger
let logger = LifecycleLogger::new(&data_dir);
let logger_rx = lifecycle_tx.subscribe();
tokio::spawn(logger.run(logger_rx));
```

- [ ] **Step 2: Spawn watchdog tick loop**

```rust
// Spawn watchdog (only ticks when BarState::Visible)
let watchdog_tx = lifecycle_tx.clone();
tokio::spawn(async move {
    let mut state = WatchdogState::new();
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        if !current_bar_state.load().watchdog_active() {
            continue;
        }
        // Health check and emit events
        let event = LifecycleEvent::WatchdogTick { healthy: true };
        watchdog_tx.send(TimestampedEvent::now(event)).ok();
    }
});
```

- [ ] **Step 3: Wire BarState actor (message channel)**

Create a `tokio::sync::mpsc` channel for BarCommand messages. The actor loop receives commands, transitions state, and triggers window recreation on recovery.

- [ ] **Step 4: Verify watchdog logs are written**

Run app, wait a few seconds, check `~/.codexbar-zoey/logs/` for a lifecycle log file.

- [ ] **Step 5: Commit**

```powershell
git add -A
git commit -m "feat: integrate watchdog and lifecycle logger into app"
```

---

### Task 3.3: Implement TrayIcon with simplified menu

**Files:**
- Modify: Tray icon setup code (find via R-3 research)

- [ ] **Step 1: Simplify tray menu to 4 items**

- `Detail 열기` → toggle detail window
- `Bar 표시 토글` → send Show/Hide command to BarState actor
- `Watchdog 재시도` → send Retry command (disabled unless Paused)
- `종료` → send Quit command

- [ ] **Step 2: Remove dynamic meter icon drawing**

Replace with static logo icon.

- [ ] **Step 3: Verify tray menu works**

Run app, right-click tray icon, verify all menu items.

- [ ] **Step 4: Commit**

```powershell
git add -A
git commit -m "feat: simplify tray icon to static logo + 4-item menu"
```

---

## Phase 4: React UI — FloatBar

### Task 4.1: Rewrite FloatBar component (ultra-minimal)

**Files:**
- Rewrite: `apps/desktop-tauri/src/floatbar/` (entire directory)
- Reference: `docs/design/floatbar-mockup.html`, `docs/design/design-notes.md`

- [ ] **Step 1: Create FloatBar.tsx with provider pills**

Use design tokens from `design-notes.md`:
- Background: `#14181C` at 74% opacity + backdrop-blur(24px)
- Border radius: 12px
- 3 provider icons (SVG circles with provider hue) + percentage text
- Font: JetBrains Mono 13px for numbers

```tsx
// apps/desktop-tauri/src/floatbar/FloatBar.tsx
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import { ProviderPill } from './ProviderPill';
import { UsageSnapshot } from '../types/usage';
import './floatbar.css';

export function FloatBar() {
  const [snapshots, setSnapshots] = useState<Map<string, UsageSnapshot>>(new Map());

  useEffect(() => {
    const unlisten = listen<UsageSnapshot>('usage:update', (event) => {
      setSnapshots(prev => new Map(prev).set(event.payload.provider, event.payload));
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0) {
      // Will be handled by drag/click logic
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    // Show context menu
  };

  return (
    <div
      className="floatbar"
      onMouseDown={handleMouseDown}
      onContextMenu={handleContextMenu}
    >
      {['claude', 'codex', 'copilot'].map(id => (
        <ProviderPill
          key={id}
          providerId={id}
          snapshot={snapshots.get(id)}
        />
      ))}
    </div>
  );
}
```

- [ ] **Step 2: Create ProviderPill.tsx with hover tooltip**

```tsx
// apps/desktop-tauri/src/floatbar/ProviderPill.tsx
import { useState, useRef } from 'react';
import { UsageSnapshot } from '../types/usage';

interface Props {
  providerId: string;
  snapshot?: UsageSnapshot;
}

const PROVIDER_COLORS: Record<string, string> = {
  claude: '#C97A3E',
  codex: '#4A9EFF',
  copilot: '#7B61FF',
};

export function ProviderPill({ providerId, snapshot }: Props) {
  const [showTooltip, setShowTooltip] = useState(false);
  const timerRef = useRef<number>();

  const handleMouseEnter = () => {
    timerRef.current = window.setTimeout(() => setShowTooltip(true), 250);
  };

  const handleMouseLeave = () => {
    clearTimeout(timerRef.current);
    setShowTooltip(false);
  };

  const pct = snapshot?.remaining_pct;
  const color = PROVIDER_COLORS[providerId] || '#6B7480';
  const isDim = !snapshot || snapshot.status === 'auth_expired';
  const isWarn = snapshot?.status === 'rate_limited';

  const displayPct = pct != null ? `${Math.round(pct)}%` : '—';

  return (
    <div
      className={`provider-pill ${isDim ? 'dim' : ''} ${isWarn ? 'warn' : ''}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <svg width="12" height="12" viewBox="0 0 12 12">
        <circle cx="6" cy="6" r="5" fill={isDim ? '#4B525A' : color} />
      </svg>
      <span className="pct">{displayPct}</span>

      {showTooltip && snapshot && (
        <div className="tooltip">
          <div className="tooltip-provider">{providerId}</div>
          {snapshot.plan && <div className="tooltip-plan">{snapshot.plan}</div>}
          {snapshot.reset_at && (
            <div className="tooltip-reset">
              리셋: {formatTimeUntil(snapshot.reset_at)}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function formatTimeUntil(isoDate: string): string {
  const diff = new Date(isoDate).getTime() - Date.now();
  if (diff <= 0) return 'now';
  const hours = Math.floor(diff / 3600000);
  const minutes = Math.floor((diff % 3600000) / 60000);
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}
```

- [ ] **Step 3: Create floatbar.css with design tokens**

```css
/* apps/desktop-tauri/src/floatbar/floatbar.css */
.floatbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  background: rgba(20, 24, 28, 0.74);
  backdrop-filter: blur(24px) saturate(140%);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  font-family: 'JetBrains Mono', monospace;
  user-select: none;
  -webkit-app-region: no-drag;
}

.provider-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  position: relative;
  cursor: default;
}

.provider-pill .pct {
  font-size: 13px;
  color: #E8EAED;
  font-weight: 500;
}

.provider-pill.dim .pct {
  color: #4B525A;
}

.provider-pill.warn .pct {
  color: #F5A623;
}

.tooltip {
  position: absolute;
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  background: rgba(28, 33, 38, 0.96);
  backdrop-filter: blur(20px);
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 12px;
  color: #A3ACB4;
  white-space: nowrap;
  z-index: 1000;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.tooltip-provider {
  font-weight: 600;
  color: #E8EAED;
  margin-bottom: 4px;
  text-transform: capitalize;
}

.tooltip-plan {
  color: #6B7480;
  font-size: 11px;
}

.tooltip-reset {
  margin-top: 4px;
  color: #A3ACB4;
}
```

- [ ] **Step 4: Create types/usage.ts**

```typescript
// apps/desktop-tauri/src/types/usage.ts
export interface UsageSnapshot {
  provider: 'claude' | 'codex' | 'copilot';
  plan: string | null;
  unit: 'token' | 'request' | 'percent';
  used: number | null;
  limit: number | null;
  remaining_pct: number | null;
  reset_at: string | null; // ISO 8601
  status: 'ok' | 'stale' | 'auth_expired' | 'rate_limited' | 'network' | 'unknown';
  last_success_at: string;
  confidence: 'high' | 'cached' | 'inferred';
}
```

- [ ] **Step 5: Verify floatbar renders (dev mode)**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri
npm run tauri dev
```

- [ ] **Step 6: Commit**

```powershell
git add -A
git commit -m "feat: rewrite FloatBar UI to ultra-minimal design"
```

---

### Task 4.2: Implement drag/click logic

**Files:**
- Modify: `apps/desktop-tauri/src/floatbar/FloatBar.tsx`
- Create: `apps/desktop-tauri/src/floatbar/useDragOrClick.ts`

- [ ] **Step 1: Create useDragOrClick hook**

```typescript
// apps/desktop-tauri/src/floatbar/useDragOrClick.ts
import { useRef, useCallback } from 'react';
import { appWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';

const DRAG_THRESHOLD = 5; // pixels

export function useDragOrClick() {
  const startPos = useRef<{ x: number; y: number } | null>(null);
  const isDragging = useRef(false);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return; // left button only
    startPos.current = { x: e.screenX, y: e.screenY };
    isDragging.current = false;
  }, []);

  const onMouseMove = useCallback((e: React.MouseEvent) => {
    if (!startPos.current) return;
    const dx = Math.abs(e.screenX - startPos.current.x);
    const dy = Math.abs(e.screenY - startPos.current.y);
    if (dx >= DRAG_THRESHOLD || dy >= DRAG_THRESHOLD) {
      isDragging.current = true;
      startPos.current = null;
      appWindow.startDragging();
    }
  }, []);

  const onMouseUp = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    if (!isDragging.current && startPos.current) {
      // This was a click, not a drag
      invoke('toggle_detail');
    }
    startPos.current = null;
    isDragging.current = false;
  }, []);

  return { onMouseDown, onMouseMove, onMouseUp };
}
```

- [ ] **Step 2: Integrate hook into FloatBar.tsx**

Replace the `handleMouseDown` with the hook:
```tsx
const { onMouseDown, onMouseMove, onMouseUp } = useDragOrClick();

return (
  <div
    className="floatbar"
    onMouseDown={onMouseDown}
    onMouseMove={onMouseMove}
    onMouseUp={onMouseUp}
    onContextMenu={handleContextMenu}
  >
    ...
  </div>
);
```

- [ ] **Step 3: Verify drag and click both work**

- Drag bar > 5px → window moves
- Click without moving → detail window toggles

- [ ] **Step 4: Commit**

```powershell
git add -A
git commit -m "feat: implement drag-or-click with 5px threshold"
```

---

## Phase 5: React UI — DetailView

### Task 5.1: Rewrite DetailView shell + provider toggle

**Files:**
- Rewrite: `apps/desktop-tauri/src/surfaces/` (or equivalent detail view directory)
- Reference: `docs/design/detail-mockup.html`

- [ ] **Step 1: Create DetailView.tsx shell**

3-card layout with provider toggle header. Use design tokens from `design-notes.md`:
- Window background: `#101418`
- Card background: `#14181C`

```tsx
// apps/desktop-tauri/src/surfaces/DetailView.tsx
import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { PlanRemainingCard } from './cards/PlanRemainingCard';
import { SessionsCard } from './cards/SessionsCard';
import { NextResetCard } from './cards/NextResetCard';
import { UsageSnapshot } from '../types/usage';
import './detail.css';

type ProviderId = 'claude' | 'codex' | 'copilot';

export function DetailView() {
  const [snapshots, setSnapshots] = useState<Map<string, UsageSnapshot>>(new Map());
  const [activeProviders, setActiveProviders] = useState<Set<ProviderId>>(
    new Set(['claude', 'codex', 'copilot'])
  );

  useEffect(() => {
    const unlisten = listen<UsageSnapshot>('usage:update', (event) => {
      setSnapshots(prev => new Map(prev).set(event.payload.provider, event.payload));
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  const toggleProvider = (id: ProviderId) => {
    setActiveProviders(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        if (next.size > 1) next.delete(id); // keep at least 1 active
      } else {
        next.add(id);
      }
      return next;
    });
  };

  return (
    <div className="detail-view">
      <header className="detail-header">
        <h1>codexbar-zoey</h1>
        <div className="provider-toggles">
          {(['claude', 'codex', 'copilot'] as ProviderId[]).map(id => (
            <button
              key={id}
              className={`toggle ${activeProviders.has(id) ? 'active' : ''}`}
              onClick={() => toggleProvider(id)}
            >
              {id}
            </button>
          ))}
        </div>
      </header>
      <main className="detail-cards">
        <PlanRemainingCard providers={activeProviders} snapshots={snapshots} />
        <SessionsCard providers={activeProviders} />
        <NextResetCard providers={activeProviders} snapshots={snapshots} />
      </main>
    </div>
  );
}
```

- [ ] **Step 2: Create placeholder card components**

Create `PlanRemainingCard.tsx`, `SessionsCard.tsx`, `NextResetCard.tsx` with empty state messages.

- [ ] **Step 3: Create detail.css**

- [ ] **Step 4: Verify detail view opens on bar click**

- [ ] **Step 5: Commit**

```powershell
git add -A
git commit -m "feat: rewrite DetailView shell with provider toggle"
```

---

### Task 5.2: Implement Card 1 — Plan Remaining chart

**Files:**
- Create: `apps/desktop-tauri/src/surfaces/cards/PlanRemainingCard.tsx`

- [ ] **Step 1: Install chart library (per R-5 result)**

```powershell
cd E:\04_Dev\02_tool\codexbar-zoey\apps\desktop-tauri
npm install recharts
```

- [ ] **Step 2: Implement time-series chart component**

Line chart showing remaining_pct over time per provider. Y-axis 0-100%. Dashed line for burn-rate projection.

- [ ] **Step 3: Wire to history data (invoke Rust command)**

Create a Tauri command `get_usage_history` that reads from `~/.codexbar-zoey/history/{provider}.jsonl`.

- [ ] **Step 4: Verify chart renders with mock data**

- [ ] **Step 5: Commit**

```powershell
git add -A
git commit -m "feat: implement Plan Remaining chart (Card 1)"
```

---

### Task 5.3: Implement Card 2 — Sessions log

**Files:**
- Create: `apps/desktop-tauri/src/surfaces/cards/SessionsCard.tsx`

- [ ] **Step 1: Implement sessions list table**

Columns: time, model, tokens, directory. Data from `~/.codexbar-zoey/sessions/{provider}.jsonl` (if R-2 found sources).

- [ ] **Step 2: Handle empty/unavailable state**

If R-2 determined sessions aren't available for a provider, show "Session log not available for this provider".

- [ ] **Step 3: Commit**

```powershell
git add -A
git commit -m "feat: implement Sessions log card (Card 2)"
```

---

### Task 5.4: Implement Card 3 — Next Reset

**Files:**
- Create: `apps/desktop-tauri/src/surfaces/cards/NextResetCard.tsx`

- [ ] **Step 1: Show reset time per provider**

For each active provider, show:
- Provider name + icon
- Next reset datetime
- Countdown (hours:minutes)
- Visual timeline/progress bar for current period

- [ ] **Step 2: Handle missing reset_at**

Show "Sign in to see reset times" with button to settings.

- [ ] **Step 3: Commit**

```powershell
git add -A
git commit -m "feat: implement Next Reset card (Card 3)"
```

---

## Phase 6: Integration & Polish

### Task 6.1: Wire usage polling → UI event emission

**Files:**
- Modify: Rust polling code (existing `status.rs` + our backoff)

- [ ] **Step 1: Add backoff to each provider's poll loop**

Integrate `BackoffPolicy` from Task 2.6 into the existing polling mechanism.

- [ ] **Step 2: Emit `usage:update` Tauri event with UsageSnapshot**

After each successful poll, emit to frontend.

- [ ] **Step 3: Save to usage-cache.json on each update**

For instant display on app restart.

- [ ] **Step 4: Append to history JSONL for Card 1**

- [ ] **Step 5: Verify end-to-end: poll → bar shows real data**

- [ ] **Step 6: Commit**

```powershell
git add -A
git commit -m "feat: wire usage polling with backoff to UI events"
```

---

### Task 6.2: First-run experience

**Files:**
- Modify: FloatBar and DetailView

- [ ] **Step 1: Detect no credentials → show dim placeholders in bar**

- [ ] **Step 2: Auto-open Detail view with auth guidance**

Filter existing Win-CodexBar auth UI to show only 3 providers.

- [ ] **Step 3: Verify first-run flow**

- [ ] **Step 4: Commit**

```powershell
git add -A
git commit -m "feat: first-run experience with auth guidance"
```

---

### Task 6.3: Bundle fonts

**Files:**
- Add: Font files to assets
- Modify: CSS to use local fonts

- [ ] **Step 1: Download Pretendard and JetBrains Mono**

Add woff2 files to `apps/desktop-tauri/src/assets/fonts/`.

- [ ] **Step 2: Update CSS @font-face declarations**

Replace Google Fonts URL with local references.

- [ ] **Step 3: Commit**

```powershell
git add -A
git commit -m "feat: bundle Pretendard and JetBrains Mono fonts locally"
```

---

### Task 6.4: Manual verification checklist

Run through the full MVP DoD checklist from spec §9:

- [ ] 3 provider 잔여 % 정확히 표시
- [ ] hover 시 리셋 시간 정확
- [ ] click으로 Detail 열리고 3 카드 데이터 표시
- [ ] 5px 이상 drag로 옮긴 위치가 재시작 후 유지
- [ ] 멀티 모니터에서 의도한 모니터에 표시, 모니터 제거 시 primary로 복귀
- [ ] 일반 창 모드 앱 위에서 bar 유지 또는 watchdog 복구
- [ ] Sleep/wake 후 정상 동작
- [ ] 트레이 `Bar 표시 토글`로 숨김 → watchdog가 다시 띄우지 않음
- [ ] 1주일 사용 후 lifecycle log 패턴 확인 (장기)

---

## Dependency Graph

```
Phase 0 (Setup + Research: R1-R5)
    ↓
Phase 1 (Provider Cleanup) — depends on R-3
    ↓
Phase 2 (Rust Core) — partially depends on R-4 for watchdog API details
    ↓
Phase 3 (Tauri Window) — depends on Phase 2 modules
    ↓
Phase 4 (FloatBar UI) — depends on Phase 3 for window to render in
    ↓
Phase 5 (DetailView UI) — depends on R-5 (chart lib), R-1/R-2 (data fields)
    ↓
Phase 6 (Integration) — ties everything together
```

Note: Phase 2 tasks (state machine, logger, backoff, window state) are **independent of each other** and can be parallelized.
