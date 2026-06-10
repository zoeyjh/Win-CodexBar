# Remove Dead Settings Options — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove 8 settings options from the UI and backend that are displayed but have no functional implementation.

**Architecture:** Bottom-up removal — start from the shared Rust crate (settings struct fields), propagate to Tauri shell bridge/commands, then clean up the React frontend types and components. The `serde(default)` deserialization ensures existing `settings.json` files with these fields won't break (unknown fields are silently ignored).

**Tech Stack:** Rust (codexbar shared crate + Tauri shell), TypeScript/React (frontend), CSS

---

## Fields Being Removed

| Field (Rust) | Field (TS) | Location |
|---|---|---|
| `tray_icon_mode` | `trayIconMode` | Display tab |
| `switcher_shows_icons` | `switcherShowsIcons` | Display tab |
| `menu_bar_shows_highest_usage` | `menuBarShowsHighestUsage` | Display tab |
| `menu_bar_shows_percent` | `menuBarShowsPercent` | Display tab |
| `menu_bar_display_mode` | `menuBarDisplayMode` | Display tab |
| `disable_keychain_access` | `disableKeychainAccess` | Advanced tab |
| `show_debug_settings` | `showDebugSettings` | Advanced tab |
| Provider-level `avoid_keychain_prompts` | `claudeAvoidKeychainPrompts` | Advanced tab |

Also removed: `TrayIconMode` enum, `MenuBarDisplayMode` TS type, `TrayIconMode` TS type, the "Menu Bar" UI section, the "KEYCHAIN ACCESS" UI section, and the "Show Debug Settings" toggle.

## File Map

**Shared Crate (`rust/src/`):**
- Modify: `rust/src/settings.rs` — remove struct fields and defaults
- Modify: `rust/src/settings/types.rs` — remove `TrayIconMode` enum and `avoid_keychain_prompts` from `ProviderConfig`
- Modify: `rust/src/settings/raw.rs` — remove from serialization struct and conversion impl
- Modify: `rust/src/settings/tests.rs` — update test JSON fixtures

**Tauri Shell (`apps/desktop-tauri/src-tauri/src/`):**
- Modify: `apps/desktop-tauri/src-tauri/src/commands/bridge.rs` — remove from `SettingsSnapshot` struct and `From<Settings>` impl
- Modify: `apps/desktop-tauri/src-tauri/src/commands/settings.rs` — remove from `SettingsUpdate` struct and apply methods

**Frontend (`apps/desktop-tauri/src/`):**
- Modify: `apps/desktop-tauri/src/types/bridge.ts` — remove fields and types
- Modify: `apps/desktop-tauri/src/surfaces/settings/tabs/DisplayTab.tsx` — remove Menu Bar section
- Modify: `apps/desktop-tauri/src/surfaces/settings/tabs/AdvancedTab.tsx` — remove Keychain section + Show Debug toggle
- Modify: `apps/desktop-tauri/src/surfaces/PopOutPanel.test.tsx` — remove fields from mock settings
- Modify: `apps/desktop-tauri/src/surfaces/TrayPanel.test.tsx` — remove fields from mock settings
- Modify: `apps/desktop-tauri/src/surfaces/settings/tabs/AboutTab.test.tsx` — remove fields from mock settings

**Locale files:**
- Modify: `rust/src/locale.rs` — remove locale key enum variants
- Modify: `rust/src/locale/keys.rs` — remove key-to-string mappings
- Modify: `rust/src/locale/english.rs` — remove English translations
- Modify: `rust/src/locale/chinese.rs` — remove Chinese translations

---

### Task 1: Remove fields from shared Rust settings struct

**Files:**
- Modify: `rust/src/settings.rs:63-68,70-80,94-95,110-116,254-257,262,266-267`
- Modify: `rust/src/settings/types.rs` — `TrayIconMode` enum and `ProviderConfig.avoid_keychain_prompts`

- [ ] **Step 1: Remove fields from `Settings` struct in `rust/src/settings.rs`**

Remove these fields from the struct (around lines 63-116):
```rust
// DELETE these lines:
    pub tray_icon_mode: TrayIconMode,
    pub switcher_shows_icons: bool,
    pub menu_bar_shows_highest_usage: bool,
    pub menu_bar_shows_percent: bool,
    pub menu_bar_display_mode: String,
    pub show_debug_settings: bool,
    pub disable_keychain_access: bool,
```

And remove from `Default` impl (around lines 254-267):
```rust
// DELETE these lines from Default::default():
            tray_icon_mode: TrayIconMode::default(),
            switcher_shows_icons: true,
            menu_bar_shows_highest_usage: false,
            menu_bar_shows_percent: false,
            menu_bar_display_mode: "detailed".to_string(),
            show_debug_settings: false,
            disable_keychain_access: false,
```

- [ ] **Step 2: Remove `TrayIconMode` enum from `rust/src/settings/types.rs`**

Remove the `TrayIconMode` enum definition and its `Display`/`Default` impls.

- [ ] **Step 3: Remove `avoid_keychain_prompts` from `ProviderConfig` in `rust/src/settings/types.rs`**

In the `ProviderConfig` struct, remove:
```rust
    pub avoid_keychain_prompts: bool,
```

- [ ] **Step 4: Remove keychain helper methods from `rust/src/settings.rs`**

Remove these methods from `impl Settings`:
```rust
    pub fn avoid_keychain_prompts(&self, id: ProviderId) -> bool { ... }
    pub fn set_avoid_keychain_prompts(&mut self, id: ProviderId, value: bool) { ... }
    pub fn claude_avoid_keychain_prompts(&self) -> bool { ... }
    pub fn set_claude_avoid_keychain_prompts(&mut self, v: bool) { ... }
```

- [ ] **Step 5: Update `rust/src/settings/raw.rs`**

Remove from `RawSettings` struct:
```rust
    claude_avoid_keychain_prompts: Option<bool>,
    show_debug_settings: bool,
    disable_keychain_access: bool,
    // and the tray/display fields:
    tray_icon_mode: TrayIconMode,
    switcher_shows_icons: bool,
    menu_bar_shows_highest_usage: bool,
    menu_bar_shows_percent: bool,
    menu_bar_display_mode: String,
```

Remove from `From<&Settings> for RawSettings` impl the corresponding field assignments.

Remove from `From<RawSettings> for Settings` impl the corresponding field assignments and the `claude_avoid_keychain_prompts` migration logic.

- [ ] **Step 6: Update tests in `rust/src/settings/tests.rs`**

Remove references to deleted fields from test JSON fixtures and assertions. For tests that specifically test these features (like `test_legacy_per_provider_fields_migrate_into_provider_configs` which checks `claude_avoid_keychain_prompts`), remove or simplify the relevant assertions.

- [ ] **Step 7: Verify shared crate compiles**

Run: `cargo build --manifest-path rust/Cargo.toml 2>&1 | head -30`
Expected: compilation errors in downstream crate (Tauri shell) but the shared crate itself should compile.

Actually, run: `cargo check -p codexbar 2>&1 | head -40`
Expected: Success (or only warnings)

---

### Task 2: Remove from Tauri shell bridge and commands

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/commands/bridge.rs:361-381,415,436-439,454-456,834-839`
- Modify: `apps/desktop-tauri/src-tauri/src/commands/settings.rs:19-22,26,37-39,86-93,125-135,166-199,237-241`

- [ ] **Step 1: Remove fields from `SettingsSnapshot` in `commands/bridge.rs`**

Remove these fields from the struct (around lines 361-381):
```rust
    tray_icon_mode: &'static str,
    switcher_shows_icons: bool,
    menu_bar_shows_highest_usage: bool,
    menu_bar_shows_percent: bool,
    menu_bar_display_mode: String,
    claude_avoid_keychain_prompts: bool,
    disable_keychain_access: bool,
    show_debug_settings: bool,
```

- [ ] **Step 2: Remove from `From<Settings> for SettingsSnapshot` impl**

Remove these lines from the `From` impl (around lines 415-456):
```rust
    let avoid_keychain_prompts = settings.claude_avoid_keychain_prompts();
    // ...
    tray_icon_mode: tray_icon_mode_label(settings.tray_icon_mode),
    switcher_shows_icons: settings.switcher_shows_icons,
    menu_bar_shows_highest_usage: settings.menu_bar_shows_highest_usage,
    menu_bar_shows_percent: settings.menu_bar_shows_percent,
    menu_bar_display_mode: settings.menu_bar_display_mode,
    claude_avoid_keychain_prompts: avoid_keychain_prompts,
    disable_keychain_access: settings.disable_keychain_access,
    show_debug_settings: settings.show_debug_settings,
```

- [ ] **Step 3: Remove `tray_icon_mode_label` function and `parse_tray_icon_mode` in bridge/settings**

In `commands/bridge.rs`, remove the `tray_icon_mode_label` function (lines 834-839).

In `commands/settings.rs`, remove `parse_tray_icon_mode` function (lines 237-241).

- [ ] **Step 4: Remove fields from `SettingsUpdate` in `commands/settings.rs`**

Remove these fields from the `SettingsUpdate` struct:
```rust
    pub tray_icon_mode: Option<String>,
    pub switcher_shows_icons: Option<bool>,
    pub menu_bar_shows_highest_usage: Option<bool>,
    pub menu_bar_shows_percent: Option<bool>,
    pub menu_bar_display_mode: Option<String>,
    pub claude_avoid_keychain_prompts: Option<bool>,
    pub disable_keychain_access: Option<bool>,
    pub show_debug_settings: Option<bool>,
```

- [ ] **Step 5: Remove apply logic from `SettingsUpdate` methods**

In `apply_provider_settings`: remove the `tray_icon_mode` block.

In `apply_display_settings`: remove the `menu_bar_display_mode`, `switcher_shows_icons`, `menu_bar_shows_highest_usage`, `menu_bar_shows_percent` blocks.

In `apply_advanced_settings`: remove the `claude_avoid_keychain_prompts`, `disable_keychain_access`, `show_debug_settings` blocks.

- [ ] **Step 6: Verify Tauri shell compiles**

Run: `cargo check --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml 2>&1 | head -40`
Expected: Success (or only warnings)

---

### Task 3: Remove from frontend TypeScript types and components

**Files:**
- Modify: `apps/desktop-tauri/src/types/bridge.ts:12,30,171-174,181,189-191,215-218,225,233-235`
- Modify: `apps/desktop-tauri/src/surfaces/settings/tabs/DisplayTab.tsx:3,7-79`
- Modify: `apps/desktop-tauri/src/surfaces/settings/tabs/AdvancedTab.tsx:91-143`

- [ ] **Step 1: Remove types from `types/bridge.ts`**

Remove line 12: `export type TrayIconMode = "single" | "perProvider";`
Remove line 30: `export type MenuBarDisplayMode = "minimal" | "compact" | "detailed";`

From `SettingsSnapshot` interface, remove:
```typescript
  trayIconMode: TrayIconMode;
  switcherShowsIcons: boolean;
  menuBarShowsHighestUsage: boolean;
  menuBarShowsPercent: boolean;
  menuBarDisplayMode: MenuBarDisplayMode;
  claudeAvoidKeychainPrompts: boolean;
  disableKeychainAccess: boolean;
  showDebugSettings: boolean;
```

From `SettingsUpdate` interface, remove:
```typescript
  trayIconMode?: TrayIconMode;
  switcherShowsIcons?: boolean;
  menuBarShowsHighestUsage?: boolean;
  menuBarShowsPercent?: boolean;
  menuBarDisplayMode?: MenuBarDisplayMode;
  claudeAvoidKeychainPrompts?: boolean;
  disableKeychainAccess?: boolean;
  showDebugSettings?: boolean;
```

- [ ] **Step 2: Remove "Menu Bar" section from `DisplayTab.tsx`**

Remove the entire first `<section>` element with title "Menu Bar" (lines 12-80), which contains Tray Icon Mode, Show Provider Icons, Prefer Highest Usage, Show Percent in Tray, and Display Mode options.

Also remove the now-unused imports: `Select` (if no longer needed), `TrayIconMode` and `MenuBarDisplayMode` types.

Check if `Select` is still used (it's not in Menu Content section), if not, remove from import.

- [ ] **Step 3: Remove "KEYCHAIN ACCESS" section and "Show Debug Settings" from `AdvancedTab.tsx`**

Remove the entire "KEYCHAIN ACCESS" section (lines 109-143).

In the "Debug" section (lines 62-89), remove the "Show Debug Settings" Field+Toggle (lines 65-76), keeping only the "Surprise Animations" Field+Toggle.

- [ ] **Step 4: Remove unused imports from cleaned components**

In `DisplayTab.tsx`, check if `Select` import is still needed (it's not used in remaining "Menu Content" section — all remaining fields use `Toggle`). Remove `Select` from imports, remove `TrayIconMode` and `MenuBarDisplayMode` from type imports.

In `AdvancedTab.tsx`, the remaining code doesn't use `disableKeychainAccess` or `claudeAvoidKeychainPrompts` from settings — no type import changes needed (props are generic).

---

### Task 4: Update test mocks

**Files:**
- Modify: `apps/desktop-tauri/src/surfaces/PopOutPanel.test.tsx`
- Modify: `apps/desktop-tauri/src/surfaces/TrayPanel.test.tsx`
- Modify: `apps/desktop-tauri/src/surfaces/settings/tabs/AboutTab.test.tsx`

- [ ] **Step 1: Remove deleted fields from mock settings objects**

In each test file, find the mock `settings` object and remove:
```typescript
    switcherShowsIcons: ...,
    menuBarShowsHighestUsage: ...,
    menuBarShowsPercent: ...,
    menuBarDisplayMode: ...,
    showDebugSettings: ...,
    // (and trayIconMode, claudeAvoidKeychainPrompts, disableKeychainAccess if present)
```

---

### Task 5: Remove locale keys

**Files:**
- Modify: `rust/src/locale.rs`
- Modify: `rust/src/locale/keys.rs`
- Modify: `rust/src/locale/english.rs`
- Modify: `rust/src/locale/chinese.rs`

- [ ] **Step 1: Remove enum variants from `rust/src/locale.rs`**

Remove these `LocaleKey` variants:
```rust
    AvoidKeychainPromptsLabel,
    AvoidKeychainPromptsHelper,
    DisableAllKeychainLabel,
    DisableAllKeychainHelper,
    ProviderClaudeAvoidKeychainPrompts,
    ProviderClaudeAvoidKeychainPromptsHelp,
```

Also check for and remove any keys related to:
- TrayIconMode (TrayIconModeLabel, TrayIconModeHelper, TrayIconModeSingle, TrayIconModePerProvider)
- ShowProviderIcons, ShowProviderIconsHelper
- PreferHighestUsage, PreferHighestUsageHelper
- ShowPercentInTray, ShowPercentInTrayHelper
- DisplayModeLabel, DisplayModeHelper, DisplayModeDetailed, DisplayModeCompact, DisplayModeMinimal
- ShowDebugSettingsLabel, ShowDebugSettingsHelper

- [ ] **Step 2: Remove key-to-string mappings from `rust/src/locale/keys.rs`**

Remove the corresponding `(LocaleKey::Variant, "StringId")` entries.

- [ ] **Step 3: Remove English translations from `rust/src/locale/english.rs`**

Remove the `LocaleKey::Variant => "translation"` entries for all deleted keys.

- [ ] **Step 4: Remove Chinese translations from `rust/src/locale/chinese.rs`**

Remove the `LocaleKey::Variant => "translation"` entries for all deleted keys.

---

### Task 6: Final verification

- [ ] **Step 1: Full Rust compilation check**

Run: `cargo build --manifest-path rust/Cargo.toml 2>&1`
Expected: Success

Run: `cargo build --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml 2>&1`
Expected: Success

- [ ] **Step 2: Run Rust tests**

Run: `cargo test --manifest-path rust/Cargo.toml 2>&1`
Expected: All tests pass

Run: `cargo test --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml 2>&1`
Expected: All tests pass

- [ ] **Step 3: Check frontend types compile**

Run: `cd apps/desktop-tauri && npx tsc --noEmit 2>&1 | head -40`
Expected: No type errors related to removed fields

- [ ] **Step 4: Run frontend tests**

Run: `cd apps/desktop-tauri && pnpm test 2>&1`
Expected: All tests pass

- [ ] **Step 5: Lint check**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | head -40`
Expected: No errors

- [ ] **Step 6: Commit**

```powershell
git add -A
git commit -m "Remove dead settings: tray-icon-mode, keychain, debug, menu-bar display options

Remove 8 settings that were persisted but had no functional implementation:
- Tray Icon Mode (Per Provider never implemented in tray_bridge)
- Show Provider Icons (no consumer)
- Prefer Highest Usage (no consumer)
- Show Percent in Tray (no consumer)
- Menu Bar Display Mode (no consumer)
- Disable All Keychain Access (macOS concept, unused on Windows)
- Avoid Keychain Prompts (macOS concept, unused on Windows)
- Show Debug Settings (no debug UI gated behind this flag)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```
