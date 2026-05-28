# Detail View Positioning, Multi-Window Reset Display, and Tooltip Direction

**Date:** 2026-05-28  
**Status:** Approved  

## Problem Statement

Three UX issues in the floating bar and detail view:

1. **Inconsistent Detail View placement** — Clicking the FloatBar opens the detail view on the left monitor (primary fallback), while the tray icon opens it on the FloatBar's monitor. No consistent rule.
2. **Missing rate windows in Next Reset card** — Only one reset time shown per provider. Claude/Codex have 5H session + weekly windows that are not displayed.
3. **Tooltip always appears below** — When FloatBar is at the screen bottom (common), tooltip gets cut off or hidden behind the taskbar.

## Design Decisions

| Issue | Decision |
|-------|----------|
| Detail View position | Always open on the monitor where the mouse cursor is, at bottom-right of work area |
| Rate window display | Grouped Cards layout: one card per provider, listing all rate windows inside |
| Tooltip direction | Dynamic: above if FloatBar is in lower half of screen, below if upper half |

---

## 1. Detail View Position: Cursor-Monitor Anchoring

### Current Behavior

- `toggle_detail` Tauri command passes `None` as position.
- Falls through: tray anchor → current monitor → window center → primary monitor.
- FloatBar click has no explicit anchor, causing inconsistent placement.

### New Behavior

Both FloatBar click and tray icon click determine the monitor containing the mouse cursor and place the Detail View at the bottom-right of that monitor's work area.

### Implementation

**`apps/desktop-tauri/src-tauri/src/commands/surface.rs`**
- Add `cursor_position: Option<(i32, i32)>` parameter to `toggle_detail`.
- Pass it through to `toggle_detail_view`.

**`apps/desktop-tauri/src-tauri/src/shell/position.rs`**
- Add `cursor_anchored_popout_position(app, cursor: (i32, i32)) -> Option<(i32, i32)>`.
- Uses `monitor_placement_containing_point` to find the monitor under the cursor.
- Computes bottom-right position within work area (same math as `detail_view_anchor_position` but monitor selection based on cursor).

**`apps/desktop-tauri/src-tauri/src/shell/transition.rs`**
- `toggle_detail_view` prefers cursor-based position when provided, before falling back to existing logic.

**`apps/desktop-tauri/src-tauri/src/tray_bridge.rs`**
- On tray left-click, capture cursor position from the click event and pass to `toggle_detail_view`.

**`apps/desktop-tauri/src/floatbar/FloatBar.tsx`**
- On click, get cursor screen position via Tauri `cursorPosition` API and pass to `toggle_detail` invoke.

---

## 2. Next Reset Card: Grouped Cards with All Rate Windows

### Current Behavior

- `DetailView.tsx` listens for `usage:update` events using the old `UsageSnapshot` type (single `reset_at`).
- `NextResetCard` shows one progress bar per provider.

### New Behavior

- Switch `DetailView` to use `ProviderUsageSnapshot` from bridge types (already emitted by backend).
- `NextResetCard` renders a **card per provider**, each card listing:
  - Provider name + plan badge
  - For each window (primary, secondary, tertiary, extras): label, used%, countdown, progress bar
- Only windows that exist for the provider are shown (e.g., Copilot shows only Monthly).

### Data Flow

The backend already sends all rate windows in `ProviderUsageSnapshot`:
- `primary` + `primaryLabel` (e.g., "Session (5H)")
- `secondary` + `secondaryLabel` (e.g., "Weekly")
- `tertiary` (e.g., "Monthly" for 30-day)
- `extraRateWindows[]` (labeled extras)

No backend changes needed. Only frontend refactoring.

### Implementation

**`apps/desktop-tauri/src/surfaces/DetailView.tsx`**
- Replace `useState<Record<UsageProvider, UsageSnapshot>>` with the `useProviders()` hook (already exists in `hooks/useProviders.ts`).
- Pass `ProviderUsageSnapshot[]` to `NextResetCard`.

**`apps/desktop-tauri/src/surfaces/cards/NextResetCard.tsx`**
- Rewrite to accept `ProviderUsageSnapshot[]`.
- Render grouped card layout:
  ```
  ┌─────────────────────────────┐
  │ ● Claude                Pro │
  │ Session (5H)   45% · 2h 31m│
  │ ████████░░░░░░░░░░░░░░░░░░ │
  │ Weekly         30% · 3d 12h│
  │ ████░░░░░░░░░░░░░░░░░░░░░░ │
  └─────────────────────────────┘
  ```

**`apps/desktop-tauri/src/surfaces/detail.css`**
- Add `.reset-card__group` (card container), `.reset-card__window` (per-window row) styles.

### Countdown Formatting

Reuse `RateWindow.format_countdown()` logic on frontend:
- `< 1h` → "42m"
- `1-24h` → "2h 31m"  
- `> 24h` → "3d 12h"

---

## 3. FloatBar Tooltip Direction: Dynamic Flip

### Current Behavior

CSS: `top: calc(100% + 10px)` — always below the pill.

### New Behavior

- Determine FloatBar's vertical position relative to screen work area.
- If FloatBar center Y > 50% of work area height → tooltip above (`.floatbar__tooltip--above`).
- If FloatBar center Y ≤ 50% → tooltip below (`.floatbar__tooltip--below`).

### Implementation

**Position detection approach:**
- On FloatBar mount and on `tauri://move` window event, query window outer position and current monitor work area.
- Compute relative position and store in React state/context.
- Pass `tooltipDirection: "above" | "below"` to `ProviderPill`.

**`apps/desktop-tauri/src/floatbar/FloatBar.tsx`**
- Add `useEffect` listening to `tauri://move` events.
- Query `window.outerPosition()` and `currentMonitor()` to determine direction.
- Provide direction via prop or context to pills.

**`apps/desktop-tauri/src/floatbar/ProviderPill.tsx`**
- Accept `tooltipDirection` prop.
- Apply CSS class based on direction.

**`apps/desktop-tauri/src/floatbar/floatbar.css`**
- `.floatbar__tooltip--below`: `top: calc(100% + 10px)` (current behavior, default)
- `.floatbar__tooltip--above`: `bottom: calc(100% + 10px); top: auto;`

---

## Testing

### Unit Tests
- Position logic: test `cursor_anchored_popout_position` with mocked monitors.
- NextResetCard: snapshot tests with varying window counts.
- Tooltip direction: test direction computation logic.

### Manual Validation
- Multi-monitor: verify Detail View opens on correct monitor from both FloatBar and tray.
- Single-monitor: verify consistent bottom-right placement.
- FloatBar at screen bottom: tooltip appears above.
- FloatBar at screen top: tooltip appears below.
- Providers with different window counts render correctly in NextResetCard.

## Out of Scope

- Taskbar embedding (Windows lacks public API; fragile)
- Tooltip animation/transition effects
- Drag-to-reposition Detail View
