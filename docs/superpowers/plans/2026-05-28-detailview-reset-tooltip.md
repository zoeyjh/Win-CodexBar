# Detail View Positioning, Multi-Window Reset, Tooltip Direction — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix three UX issues — consistent Detail View placement on cursor's monitor, show all rate windows in Next Reset card, and dynamic tooltip direction.

**Architecture:** Three independent changes: (1) Rust position logic + Tauri command receives cursor coords, (2) Frontend NextResetCard rewrite using existing `ProviderUsageSnapshot` data, (3) FloatBar tooltip CSS + position detection.

**Tech Stack:** Rust/Tauri (position), React/TypeScript (frontend), CSS (tooltip)

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `apps/desktop-tauri/src-tauri/src/commands/surface.rs` | Modify | Add cursor param to `toggle_detail` |
| `apps/desktop-tauri/src-tauri/src/shell/position.rs` | Modify | Add `cursor_anchored_popout_position` |
| `apps/desktop-tauri/src-tauri/src/shell/transition.rs` | Modify | Use cursor position in `toggle_detail_view` |
| `apps/desktop-tauri/src-tauri/src/tray_bridge.rs` | Modify | Pass click position as cursor to detail view |
| `apps/desktop-tauri/src/floatbar/useDragOrClick.ts` | Modify | Pass cursor position to `toggle_detail` invoke |
| `apps/desktop-tauri/src/surfaces/DetailView.tsx` | Modify | Switch to `useProviders()` hook |
| `apps/desktop-tauri/src/surfaces/cards/NextResetCard.tsx` | Rewrite | Grouped Cards with all rate windows |
| `apps/desktop-tauri/src/surfaces/detailShared.ts` | Modify | Add helpers for ProviderUsageSnapshot |
| `apps/desktop-tauri/src/surfaces/detail.css` | Modify | Add grouped-card reset styles |
| `apps/desktop-tauri/src/floatbar/FloatBar.tsx` | Modify | Detect position, pass tooltip direction |
| `apps/desktop-tauri/src/floatbar/ProviderPill.tsx` | Modify | Accept tooltipDirection prop |
| `apps/desktop-tauri/src/floatbar/floatbar.css` | Modify | Add above/below tooltip styles |

---

## Task 1: Cursor-Anchored Detail View Position (Rust)

**Files:**
- Modify: `apps/desktop-tauri/src-tauri/src/commands/surface.rs`
- Modify: `apps/desktop-tauri/src-tauri/src/shell/position.rs`
- Modify: `apps/desktop-tauri/src-tauri/src/shell/transition.rs`
- Modify: `apps/desktop-tauri/src-tauri/src/tray_bridge.rs`

- [ ] **Step 1: Add `cursor_anchored_popout_position` to `position.rs`**

Add this function after `detail_view_anchor_position`:

```rust
/// Position the Detail View at bottom-right of the monitor containing `cursor`.
/// Falls back to `detail_view_anchor_position` if the cursor monitor cannot be resolved.
pub fn cursor_anchored_popout_position(app: &AppHandle, cursor: (i32, i32)) -> Option<(i32, i32)> {
    const DETAIL_WIDTH_LOGICAL: f64 = 480.0;
    const DETAIL_INNER_HEIGHT_LOGICAL: f64 = 460.0;
    const TITLE_BAR_LOGICAL: f64 = 32.0;
    const PADDING_LOGICAL: f64 = 12.0;

    let window = app.get_webview_window("main")?;
    let monitors = window.available_monitors().ok()?;
    let placements: Vec<_> = monitors.iter().map(monitor_placement).collect();

    let monitor = monitor_placement_containing_point(&placements, cursor.0, cursor.1)
        .or_else(|| {
            window
                .current_monitor()
                .ok()
                .flatten()
                .map(|m| monitor_placement(&m))
        })?;

    let scale = monitor.scale_factor;
    let wa = &monitor.work_area;

    let detail_w = (DETAIL_WIDTH_LOGICAL * scale) as i32;
    let detail_outer_h = ((DETAIL_INNER_HEIGHT_LOGICAL + TITLE_BAR_LOGICAL) * scale) as i32;
    let pad = (PADDING_LOGICAL * scale) as i32;

    let right = wa.x + wa.width as i32;
    let bottom = wa.y + wa.height as i32;

    let x = right - detail_w - pad;
    let y = bottom - detail_outer_h - pad;
    Some((x, y))
}
```

- [ ] **Step 2: Update `toggle_detail` command to accept cursor position**

In `apps/desktop-tauri/src-tauri/src/commands/surface.rs`, change:

```rust
#[tauri::command]
pub fn toggle_detail(app: tauri::AppHandle, cursor_x: Option<i32>, cursor_y: Option<i32>) -> Result<(), String> {
    let position = match (cursor_x, cursor_y) {
        (Some(x), Some(y)) => crate::shell::cursor_anchored_popout_position(&app, (x, y)),
        _ => None,
    };
    crate::shell::toggle_detail_view(&app, position)
}
```

- [ ] **Step 3: Export `cursor_anchored_popout_position` from shell module**

In `apps/desktop-tauri/src-tauri/src/shell/mod.rs`, add to the `pub use position::` block:

```rust
pub use position::{
    cursor_anchored_popout_position, default_surface_position, detail_view_anchor_position,
    inferred_tray_panel_position, remember_current_geometry_if_settings,
    shortcut_panel_position, tray_panel_position,
};
```

- [ ] **Step 4: Update tray bridge to use cursor position**

In `apps/desktop-tauri/src-tauri/src/tray_bridge.rs`, change the left-click handler (line ~217-219):

```rust
if button == MouseButton::Left {
    let cursor = (position.x as i32, position.y as i32);
    let pos = shell::cursor_anchored_popout_position(app, cursor)
        .or_else(|| shell::detail_view_anchor_position(app));
    let _ = shell::toggle_detail_view(app, pos);
}
```

Also update the `handle_menu_event` ToggleDetail case (~line 234-236):

```rust
Some(MenuAction::ToggleDetail) => {
    let position = shell::detail_view_anchor_position(app);
    let _ = shell::toggle_detail_view(app, position);
}
```

(Menu doesn't have cursor info, so keep existing fallback.)

- [ ] **Step 5: Build and verify Rust compiles**

Run: `cargo build --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml`
Expected: Successful compilation.

- [ ] **Step 6: Commit**

```bash
git add apps/desktop-tauri/src-tauri/src/commands/surface.rs apps/desktop-tauri/src-tauri/src/shell/position.rs apps/desktop-tauri/src-tauri/src/shell/mod.rs apps/desktop-tauri/src-tauri/src/tray_bridge.rs
git commit -m "feat: anchor Detail View to cursor's monitor"
```

---

## Task 2: Pass Cursor Position from FloatBar Click (Frontend)

**Files:**
- Modify: `apps/desktop-tauri/src/floatbar/useDragOrClick.ts`

- [ ] **Step 1: Update `useDragOrClick` to pass cursor position**

In `apps/desktop-tauri/src/floatbar/useDragOrClick.ts`, change the `onMouseUp` handler to pass screen coordinates:

```typescript
const onMouseUp = useCallback((event: React.MouseEvent<HTMLElement>) => {
  if (event.button !== 0) {
    return;
  }

  const shouldToggle = startPointRef.current !== null && !draggedRef.current;
  startPointRef.current = null;
  draggedRef.current = false;

  if (shouldToggle) {
    void invoke("toggle_detail", {
      cursorX: Math.round(event.screenX),
      cursorY: Math.round(event.screenY),
    }).catch(() => {});
  }
}, []);
```

- [ ] **Step 2: Update Rust command parameter names to match**

The Tauri command uses snake_case deserialization from camelCase by default. Verify the `toggle_detail` command parameters match:
- Frontend sends `cursorX` / `cursorY` (camelCase)
- Rust receives `cursor_x` / `cursor_y` (snake_case)

Tauri's default `#[tauri::command]` uses `rename_all = "camelCase"` for deserialization, so `cursor_x: Option<i32>` maps from `cursorX`. No additional changes needed.

- [ ] **Step 3: Commit**

```bash
git add apps/desktop-tauri/src/floatbar/useDragOrClick.ts
git commit -m "feat: pass cursor position from FloatBar click to toggle_detail"
```

---

## Task 3: Rewrite NextResetCard with Grouped Cards Layout

**Files:**
- Modify: `apps/desktop-tauri/src/surfaces/DetailView.tsx`
- Rewrite: `apps/desktop-tauri/src/surfaces/cards/NextResetCard.tsx`
- Modify: `apps/desktop-tauri/src/surfaces/detailShared.ts`
- Modify: `apps/desktop-tauri/src/surfaces/detail.css`

- [ ] **Step 1: Add countdown formatter for RateWindowSnapshot to detailShared.ts**

Append to `apps/desktop-tauri/src/surfaces/detailShared.ts`:

```typescript
import type { ProviderUsageSnapshot, RateWindowSnapshot } from "../types/bridge";

export const PROVIDER_COLORS: Record<string, string> = {
  claude: "#C97A3E",
  codex: "#4A9EFF",
  copilot: "#7B61FF",
};

export function formatWindowCountdown(resetsAt: string | null): string {
  if (!resetsAt) return "--";
  const remainingMs = new Date(resetsAt).getTime() - Date.now();
  if (Number.isNaN(remainingMs) || remainingMs <= 0) return "now";
  const totalMinutes = Math.ceil(remainingMs / 60_000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours > 24) {
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h`;
  }
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

export interface RateWindowEntry {
  label: string;
  window: RateWindowSnapshot;
}

export function collectRateWindows(snapshot: ProviderUsageSnapshot): RateWindowEntry[] {
  const windows: RateWindowEntry[] = [];
  windows.push({ label: snapshot.primaryLabel ?? "Session", window: snapshot.primary });
  if (snapshot.secondary) {
    windows.push({ label: snapshot.secondaryLabel ?? "Weekly", window: snapshot.secondary });
  }
  if (snapshot.tertiary) {
    windows.push({ label: "Monthly", window: snapshot.tertiary });
  }
  if (snapshot.modelSpecific) {
    windows.push({ label: "Model", window: snapshot.modelSpecific });
  }
  for (const extra of snapshot.extraRateWindows) {
    windows.push({ label: extra.title, window: extra.window });
  }
  return windows;
}
```

- [ ] **Step 2: Rewrite NextResetCard.tsx**

Replace the entire file `apps/desktop-tauri/src/surfaces/cards/NextResetCard.tsx`:

```tsx
import type { ProviderUsageSnapshot } from "../../types/bridge";
import { openSettingsWindow } from "../../lib/tauri";
import {
  PROVIDER_COLORS,
  collectRateWindows,
  formatWindowCountdown,
} from "../detailShared";

export default function NextResetCard({
  providers,
}: {
  providers: ProviderUsageSnapshot[];
}) {
  const activeProviders = providers.filter((p) => !p.error);

  if (activeProviders.length === 0) {
    return (
      <section className="detail-card detail-card--reset" aria-labelledby="detail-reset-heading">
        <div className="detail-card__header">
          <div>
            <h2 id="detail-reset-heading">Next Reset</h2>
            <p>Provider reset schedule</p>
          </div>
        </div>
        <div className="detail-card__empty detail-card__empty--stacked">
          <span>Sign in to see reset times</span>
          <button type="button" className="detail-card__button" onClick={() => openSettingsWindow("providers")}>
            Open Settings
          </button>
        </div>
      </section>
    );
  }

  return (
    <section className="detail-card detail-card--reset" aria-labelledby="detail-reset-heading">
      <div className="detail-card__header">
        <div>
          <h2 id="detail-reset-heading">Next Reset</h2>
          <p>Provider reset schedule</p>
        </div>
      </div>
      <div className="reset-groups">
        {activeProviders.map((provider) => {
          const color = PROVIDER_COLORS[provider.providerId] ?? "#888";
          const windows = collectRateWindows(provider);
          return (
            <div key={provider.providerId} className="reset-group">
              <div className="reset-group__header">
                <span className="reset-group__dot" style={{ backgroundColor: color }} />
                <strong className="reset-group__name">{provider.displayName}</strong>
                {provider.planName && (
                  <span className="reset-group__plan">{provider.planName}</span>
                )}
              </div>
              <div className="reset-group__windows">
                {windows.map((entry) => (
                  <div key={entry.label} className="reset-window">
                    <div className="reset-window__row">
                      <span className="reset-window__label">{entry.label}</span>
                      <span className="reset-window__info">
                        {Math.round(entry.window.usedPercent)}% used · {formatWindowCountdown(entry.window.resetsAt)}
                      </span>
                    </div>
                    <div className="reset-window__bar">
                      <span
                        className="reset-window__fill"
                        style={{
                          width: `${Math.min(100, entry.window.usedPercent)}%`,
                          backgroundColor: color,
                        }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
```

- [ ] **Step 3: Update DetailView.tsx to use `useProviders` hook**

Replace `apps/desktop-tauri/src/surfaces/DetailView.tsx`:

```tsx
import { useMemo } from "react";
import { useProviders } from "../hooks/useProviders";
import type { BootstrapState } from "../types/bridge";
import { USAGE_PROVIDERS } from "../types/usage";
import NextResetCard from "./cards/NextResetCard";
import "./detail.css";

export default function DetailView({
  state: _state,
  providerId,
}: {
  state: BootstrapState;
  providerId?: string;
}) {
  const { providers } = useProviders();

  const filteredProviders = useMemo(() => {
    if (providerId && USAGE_PROVIDERS.includes(providerId as any)) {
      return providers.filter((p) => p.providerId === providerId);
    }
    return providers;
  }, [providers, providerId]);

  return (
    <main className="detail-view">
      <section className="detail-view__grid">
        <NextResetCard providers={filteredProviders} />
      </section>
    </main>
  );
}
```

- [ ] **Step 4: Add grouped-card CSS styles to detail.css**

Append to `apps/desktop-tauri/src/surfaces/detail.css`:

```css
/* ── Reset Groups (Grouped Cards) ─────────────────────────────────── */

.reset-groups {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.reset-group {
  background: var(--detail-card-inset);
  border-radius: 8px;
  padding: 12px;
}

.reset-group__header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.reset-group__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.reset-group__name {
  font-size: 13px;
  color: var(--detail-fg-1);
}

.reset-group__plan {
  margin-left: auto;
  font-size: 11px;
  color: var(--detail-fg-3);
}

.reset-group__windows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.reset-window__row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.reset-window__label {
  font-size: 12px;
  color: var(--detail-fg-3);
}

.reset-window__info {
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--detail-fg-2);
}

.reset-window__bar {
  height: 3px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 2px;
  overflow: hidden;
}

.reset-window__fill {
  display: block;
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}
```

- [ ] **Step 5: Build frontend and verify**

Run: `cd apps/desktop-tauri && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 6: Commit**

```bash
git add apps/desktop-tauri/src/surfaces/DetailView.tsx apps/desktop-tauri/src/surfaces/cards/NextResetCard.tsx apps/desktop-tauri/src/surfaces/detailShared.ts apps/desktop-tauri/src/surfaces/detail.css
git commit -m "feat: show all rate windows in Next Reset card (grouped cards)"
```

---

## Task 4: Dynamic Tooltip Direction for FloatBar

**Files:**
- Modify: `apps/desktop-tauri/src/floatbar/FloatBar.tsx`
- Modify: `apps/desktop-tauri/src/floatbar/ProviderPill.tsx`
- Modify: `apps/desktop-tauri/src/floatbar/floatbar.css`

- [ ] **Step 1: Add position detection to FloatBar.tsx**

Update `apps/desktop-tauri/src/floatbar/FloatBar.tsx` to detect whether the bar is in the upper or lower half of the screen:

```tsx
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emitCachedUsageUpdates } from "../lib/tauri";
import type { BootstrapState } from "../types/bridge";
import { USAGE_PROVIDERS, type UsageProvider, type UsageSnapshot } from "../types/usage";
import ProviderPill from "./ProviderPill";
import { useDragOrClick } from "./useDragOrClick";
import "./floatbar.css";

export type TooltipDirection = "above" | "below";

const EMPTY_SNAPSHOTS: Record<UsageProvider, UsageSnapshot | null> = {
  claude: null,
  codex: null,
  copilot: null,
};

function useTooltipDirection(): TooltipDirection {
  const [direction, setDirection] = useState<TooltipDirection>("below");

  useEffect(() => {
    let cancelled = false;

    async function detectDirection() {
      try {
        const win = getCurrentWindow();
        const position = await win.outerPosition();
        const monitor = await win.currentMonitor();
        if (cancelled || !monitor) return;

        const workArea = monitor.size;
        const monitorY = monitor.position.y;
        const windowY = position.y;
        const relativeY = windowY - monitorY;
        const halfHeight = workArea.height / 2;

        setDirection(relativeY > halfHeight ? "above" : "below");
      } catch {
        // Fallback to below on error
      }
    }

    void detectDirection();

    const unlisten = listen("tauri://move", () => {
      void detectDirection();
    });

    return () => {
      cancelled = true;
      void unlisten.then((fn) => fn());
    };
  }, []);

  return direction;
}

export default function FloatBar({ state: _state }: { state: BootstrapState }) {
  const [snapshots, setSnapshots] = useState<Record<UsageProvider, UsageSnapshot | null>>(
    EMPTY_SNAPSHOTS,
  );
  const dragOrClick = useDragOrClick();
  const tooltipDirection = useTooltipDirection();

  useEffect(() => {
    document.body.classList.add("floatbar-window");
    document.documentElement.classList.add("floatbar-window-root");
    return () => {
      document.body.classList.remove("floatbar-window");
      document.documentElement.classList.remove("floatbar-window-root");
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<UsageSnapshot>("usage:update", (event) => {
      setSnapshots((current) => ({
        ...current,
        [event.payload.provider]: event.payload,
      }));
    });
    void emitCachedUsageUpdates().catch(() => {});

    return () => {
      void unlisten.then((stopListening) => stopListening()).catch(() => {});
    };
  }, []);

  return (
    <div
      className="floatbar"
      onContextMenu={(event) => event.preventDefault()}
      {...dragOrClick}
    >
      {USAGE_PROVIDERS.map((provider) => (
        <ProviderPill
          key={provider}
          provider={provider}
          snapshot={snapshots[provider]}
          tooltipDirection={tooltipDirection}
        />
      ))}
    </div>
  );
}
```

- [ ] **Step 2: Update ProviderPill to use tooltipDirection**

Update `apps/desktop-tauri/src/floatbar/ProviderPill.tsx` to accept and apply the direction prop:

```tsx
import { useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties } from "react";
import type { UsageProvider, UsageSnapshot } from "../types/usage";
import type { TooltipDirection } from "./FloatBar";

const PROVIDER_COLORS: Record<UsageProvider, string> = {
  claude: "#C97A3E",
  codex: "#4A9EFF",
  copilot: "#7B61FF",
};

function formatProviderName(provider: UsageProvider): string {
  return provider.charAt(0).toUpperCase() + provider.slice(1);
}

function formatPercent(snapshot: UsageSnapshot | null): string {
  if (!snapshot || snapshot.status === "auth_expired" || snapshot.remaining_pct === null) {
    return "--";
  }

  return `${Math.round(snapshot.remaining_pct)}%`;
}

function formatResetTime(resetAt: string | null): string {
  if (!resetAt) {
    return "Reset unavailable";
  }

  const resetMs = new Date(resetAt).getTime();
  if (Number.isNaN(resetMs)) {
    return "Reset unavailable";
  }

  const totalMinutes = Math.max(0, Math.ceil((resetMs - Date.now()) / 60_000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return `Resets in ${hours}h ${minutes}m`;
}

export default function ProviderPill({
  provider,
  snapshot,
  tooltipDirection = "below",
}: {
  provider: UsageProvider;
  snapshot: UsageSnapshot | null;
  tooltipDirection?: TooltipDirection;
}) {
  const [tooltipOpen, setTooltipOpen] = useState(false);
  const hoverTimerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (hoverTimerRef.current !== null) {
        window.clearTimeout(hoverTimerRef.current);
      }
    };
  }, []);

  const pillClassName = useMemo(() => {
    if (!snapshot || snapshot.status === "auth_expired") {
      return "floatbar__pill floatbar__pill--dim";
    }

    if (snapshot.status === "rate_limited") {
      return "floatbar__pill floatbar__pill--warn";
    }

    return "floatbar__pill";
  }, [snapshot]);

  const providerName = formatProviderName(provider);
  const percentage = formatPercent(snapshot);
  const tooltipReset = formatResetTime(snapshot?.reset_at ?? null);

  const handleMouseEnter = () => {
    if (hoverTimerRef.current !== null) {
      window.clearTimeout(hoverTimerRef.current);
    }

    hoverTimerRef.current = window.setTimeout(() => {
      setTooltipOpen(true);
    }, 250);
  };

  const handleMouseLeave = () => {
    if (hoverTimerRef.current !== null) {
      window.clearTimeout(hoverTimerRef.current);
      hoverTimerRef.current = null;
    }
    setTooltipOpen(false);
  };

  const tooltipClass = `floatbar__tooltip floatbar__tooltip--${tooltipDirection}`;

  return (
    <div
      className={pillClassName}
      data-provider={provider}
      data-testid={`provider-pill-${provider}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      style={{ "--provider-color": PROVIDER_COLORS[provider] } as CSSProperties}
    >
      <svg
        aria-hidden
        className="floatbar__icon"
        viewBox="0 0 12 12"
        width="12"
        height="12"
      >
        <circle cx="6" cy="6" r="5" fill="currentColor" />
      </svg>
      <span className="floatbar__percent">{percentage}</span>
      {tooltipOpen ? (
        <div className={tooltipClass} role="tooltip">
          <div className="floatbar__tooltip-title">{providerName}</div>
          {snapshot?.plan ? <div className="floatbar__tooltip-line">{snapshot.plan}</div> : null}
          <div className="floatbar__tooltip-line">{tooltipReset}</div>
        </div>
      ) : null}
    </div>
  );
}
```

- [ ] **Step 3: Update floatbar.css with directional tooltip styles**

In `apps/desktop-tauri/src/floatbar/floatbar.css`, replace the existing `.floatbar__tooltip` rule:

```css
.floatbar__tooltip {
  position: absolute;
  z-index: 1;
  display: flex;
  min-width: 180px;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  background: rgba(28, 33, 38, 0.96);
  box-shadow: 0 14px 30px rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  white-space: nowrap;
}

.floatbar__tooltip--below {
  top: calc(100% + 10px);
  bottom: auto;
  left: 0;
}

.floatbar__tooltip--above {
  bottom: calc(100% + 10px);
  top: auto;
  left: 0;
}
```

- [ ] **Step 4: Build and verify no type errors**

Run: `cd apps/desktop-tauri && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-tauri/src/floatbar/FloatBar.tsx apps/desktop-tauri/src/floatbar/ProviderPill.tsx apps/desktop-tauri/src/floatbar/floatbar.css
git commit -m "feat: dynamic tooltip direction based on FloatBar screen position"
```

---

## Task 5: Update Tests

**Files:**
- Modify: `apps/desktop-tauri/src/surfaces/DetailView.test.tsx`
- Modify: `apps/desktop-tauri/src/floatbar/FloatBar.test.tsx`

- [ ] **Step 1: Update DetailView tests for new prop shape**

The `DetailView` no longer takes `activeProviders` / `snapshots` via old types. Update the test to mock `useProviders` hook:

```tsx
// In DetailView.test.tsx, mock the useProviders hook:
vi.mock("../hooks/useProviders", () => ({
  useProviders: () => ({
    providers: [
      {
        providerId: "claude",
        displayName: "Claude",
        primary: { usedPercent: 45, remainingPercent: 55, windowMinutes: 300, resetsAt: new Date(Date.now() + 9000000).toISOString(), resetDescription: null, isExhausted: false, reservePercent: null, reserveDescription: null },
        primaryLabel: "Session (5H)",
        secondary: { usedPercent: 30, remainingPercent: 70, windowMinutes: 10080, resetsAt: new Date(Date.now() + 302400000).toISOString(), resetDescription: null, isExhausted: false, reservePercent: null, reserveDescription: null },
        secondaryLabel: "Weekly",
        modelSpecific: null,
        tertiary: null,
        extraRateWindows: [],
        cost: null,
        planName: "Pro",
        accountEmail: null,
        sourceLabel: "web",
        updatedAt: new Date().toISOString(),
        error: null,
        pace: null,
        accountOrganization: null,
        trayStatusLabel: null,
      },
    ],
    isRefreshing: false,
    refresh: () => {},
    lastRefresh: null,
    hasCachedData: true,
  }),
}));
```

Adjust existing test assertions to match the new card structure (look for `reset-group` class names instead of `reset-card__item`).

- [ ] **Step 2: Update FloatBar tests for tooltipDirection prop**

In `FloatBar.test.tsx`, if there are tests for tooltip display, ensure they still pass. The `ProviderPill` now takes `tooltipDirection`; mock `getCurrentWindow` and `currentMonitor` to avoid Tauri API errors in tests:

```tsx
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    outerPosition: () => Promise.resolve({ x: 100, y: 800 }),
    currentMonitor: () => Promise.resolve({ size: { width: 1920, height: 1080 }, position: { x: 0, y: 0 } }),
  }),
}));
```

- [ ] **Step 3: Run tests**

Run: `cd apps/desktop-tauri && npm test -- --run`
Expected: All tests pass.

- [ ] **Step 4: Run Rust tests**

Run: `cargo test --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-tauri/src/surfaces/DetailView.test.tsx apps/desktop-tauri/src/floatbar/FloatBar.test.tsx
git commit -m "test: update tests for new NextResetCard and tooltip direction"
```

---

## Task 6: Final Lint and Format

- [ ] **Step 1: Format Rust code**

Run: `cargo fmt --all`

- [ ] **Step 2: Clippy check**

Run: `cargo clippy --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml --all-targets -- -D warnings`
Expected: No warnings.

- [ ] **Step 3: Format frontend**

Run: `cd apps/desktop-tauri && npx biome check --write src/` (or the project's configured formatter)

- [ ] **Step 4: Final commit if formatting changes**

```bash
git add -A
git commit -m "chore: format and lint" --allow-empty
```
