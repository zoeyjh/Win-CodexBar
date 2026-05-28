# R-4: Tauri Window API Capabilities

## Tauri Version: v2

## Available APIs (confirmed by existing usage)
- ✅ `WebviewWindowBuilder::new()` — runtime window creation (floatbar already uses this)
- ✅ `.inner_size()` — set window size
- ✅ `.always_on_top(true)` — builder property
- ✅ `set_always_on_top(true)` — setter (runtime)
- ✅ `current_monitor()` — get monitor info
- ✅ `outer_position()` — get window position
- ✅ `is_visible()` — check visibility
- ✅ `.decorations(false)`, `.shadow(false)`, `.resizable(false)`, `.skip_taskbar(true)`

## NOT Available
- ❌ `is_always_on_top()` — NO getter exists. Watchdog must track this via last setter call timestamp.

## Current FloatBar Window Creation
Already runtime-built in `src/floatbar/window.rs:46-80`:
```rust
WebviewWindowBuilder::new(...)
    .inner_size(...)
    .decorations(false)
    .shadow(false)
    .resizable(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .build()
```

## Existing Recovery/Watchdog
- None. No active monitoring of bar window health.
- Geometry persistence exists on move/resize/close (src/floatbar/mod.rs:41-45)
- Shell transition has recovery for window property failures

## Impact on Watchdog Design
- `is_always_on_top()` fallback: track last `set_always_on_top(true)` call time. If >60s since last assertion, re-assert.
- `is_visible()`: available, use directly
- Position check: `outer_position()` + `current_monitor()` for work area bounds
- Handle validity: check `app.get_webview_window("floatbar").is_some()`
