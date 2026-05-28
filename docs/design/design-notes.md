# codexbar-zoey · Design Notes

> Visual specs for the two mockups in this folder.
> Target: **Windows 11, dark mode**, minimal, information-dense.

- `floatbar-mockup.html` — § 5.1 FloatBar (5 states + hover tooltip)
- `detail-mockup.html` — § 5.5 DetailView (3 cards + provider toggle + empty states)

---

## 1. Color palette

All values are **HEX** (or `rgba` where noted). Tokens are grouped by role.

### Surfaces

| Token | Value | Used for |
|---|---|---|
| `bg / page`    | `#0A0D10` | Desktop/page backdrop |
| `bg / window`  | `#101418` | Detail window fill |
| `bg / card`    | `#14181C` | Cards, panels |
| `bg / card-2`  | `#181D22` | Inset cards (reset items, swatches) |
| `bar / acrylic`| `#14181C` @ **74 %** + `backdrop-blur(24px) saturate(140%)` | Floating bar |
| `tooltip`      | `#1C2126` @ **96 %** + `backdrop-blur(20px)` | Hover tooltips |

### Foreground (text)

| Token | Value | Used for |
|---|---|---|
| `fg / 1`   | `#E8EAED` | Primary text, headings, percentages |
| `fg / 2`   | `#A3ACB4` | Secondary body text |
| `fg / 3`   | `#6B7480` | Meta, labels, eyebrows |
| `fg / dim` | `#4B525A` | Auth-expired, disabled, placeholders |

### Status (semantic — same meaning everywhere)

| Token | Value | Meaning |
|---|---|---|
| `OK`      | `#8DC63F` | Healthy ( ≥ 20 % remaining ) — also the brand accent |
| `warn`    | `#F5A623` | Low ( < 20 % ), rate-limited, projected to hit limit |
| `error`   | `#F04438` | Critical ( < 10 % ), auth failure |
| `dim`     | `#4B525A` | Auth-expired, no data |

### Provider hues (chart & reset card only)

Used **only** for line/marker discrimination in the chart and the colored icon in the reset card. **Never** to encode status.

| Provider | Value |
|---|---|
| Claude   | `#C97A3E` |
| Codex    | `#8DC63F` |
| Copilot  | `#7B9CFF` |

### Borders & dividers

| Token | Value |
|---|---|
| `border / subtle` | `rgba(255, 255, 255, 0.06)` |
| `border / default`| `rgba(255, 255, 255, 0.07)` |
| `border / strong` | `rgba(255, 255, 255, 0.10)` |
| `grid-line` (chart) | `rgba(255, 255, 255, 0.04)` |

### Shadows

| Token | Value |
|---|---|
| `bar` | `0 12px 32px rgba(0,0,0,0.55)`, `0 1px 0 rgba(255,255,255,0.05) inset` |
| `tooltip` | `0 14px 30px rgba(0,0,0,0.60)` |
| `window` | `0 24px 48px rgba(0,0,0,0.45)`, `0 1px 0 rgba(255,255,255,0.04) inset` |

---

## 2. Typography

| Use | Family | Size / weight |
|---|---|---|
| UI sans     | **Pretendard** (`'Pretendard Variable'`, fallback `Segoe UI Variable`, `Segoe UI`) | 11 – 14 px / 500 – 600 |
| Mono / numeric | **JetBrains Mono** (fallback `Cascadia Code`, `ui-monospace`) | 10 – 12 px / 400 – 500 |
| Eyebrow / section header | Pretendard | 10 px, 600, letter-spacing 0.18 em, uppercase |
| Bar percentage | Pretendard | **12 px / 600**, `tabular-nums` |
| H3 (card title) | Pretendard | 12.5 px / 600 |

`font-feature-settings: "tnum" on` everywhere — keeps numbers in fixed-width columns.

---

## 3. Floating Bar — dimensions

| Property | Value |
|---|---|
| Height | **32 px** |
| Min width | **~228 px** (auto-sizes to content) |
| Padding (x) | **12 px** |
| Item gap | **14 px** |
| Item internal gap (icon ↔ pct) | 6 px |
| Icon size | **14 × 14 px** |
| Provider separator | 1 px, `rgba(255,255,255,0.06)`, 14 px tall |
| Radius | **10 px** |
| Border | 1 px, `rgba(255,255,255,0.07)` |
| Font | 12 px / 600, tabular-nums |

### Tooltip

| Property | Value |
|---|---|
| Offset from bar | 10 px |
| Padding | 10 × 12 px |
| Min width | 200 px |
| Radius | 8 px |
| Font | 11.5 px / line-height 1.5 |
| Arrow | 9 px diamond, 16 px from left edge |
| Show delay | 250 ms hover-in, 0 ms hover-out |

---

## 4. Detail View — dimensions

| Property | Value |
|---|---|
| Window | **1180 × 640** |
| Win 11 title bar | 32 px |
| App header (toggle row) | 56 px |
| Body padding | 14 px |
| Card gap | 14 px |
| Card radius | 10 px |
| Card padding | 16 × 18 px |
| H3 size | 12.5 px / 600 |
| Body text | 11 px / 1.5 |
| Mono columns | 10.5 px |

Layout grid:

```
┌──────────────────────────────────────────────────────────┐
│   Card 1 — Plan remaining (chart, full width)            │ 240 px
├──────────────────────────────────────────┬───────────────┤
│   Card 2 — Sessions (table)              │  Card 3 —     │
│                                          │  Next reset   │ 264 px
└──────────────────────────────────────────┴───────────────┘
   1 fr                                      360 px
```

---

## 5. Design decisions — rationale

### Status uses one color, providers use another axis

On the bar, **all three providers share the same green when healthy** (`#8DC63F`).
Color only shifts when state shifts: amber → low, red → critical, gray → auth-expired.
The eye scans for any *non-green* and that always means "look here."
Per-provider brand colors would dilute that signal.

Provider hues exist (orange / green / blue) but live **only** inside the chart
and the reset card, where multiple lines/items need to be visually distinguished
from each other. There, hue identifies *which provider*; saturation/value never encodes status.

### 32 px bar

Tighter than the Win 11 taskbar (48 px) so the bar reads as an ambient HUD,
not a second taskbar. Three providers fit in ~228 px — a footprint that
disappears against any 1440 px-wide app.

### Acrylic, not opaque

74 % fill + 24 px backdrop blur matches the Win 11 Mica feel. The bar inherits
whatever's behind it — never looks "stuck on" a dark IDE or a bright browser.

### Reset time stays off the bar

Per spec, % changes every poll (30 – 120 s); reset times change hourly at most.
Constant info goes on the bar, occasional info goes in the tooltip on hover.
Less to ignore.

### Auth-expired dims, doesn't hide

A disappearing slot would make the bar feel *broken* — the exact issue we're
trying to fix. Instead: opacity `0.55` + `— —` placeholder. Layout stays stable,
status is unambiguous.

### Chart axis is "remaining %", not "used %"

Down = depleting. The chart's axis is the **same number** that shows up on the
bar, the toggle chips, and the reset card. Lines start at 100 % when the cycle
resets and trend down; hitting the bottom = empty. A dashed projection extends
each line at the current burn rate so "will I hit the limit before reset?" is
visible at a glance.

### Toggle group is multi-select, all-on by default

The point of the Detail view is **comparison**. Tapping a chip dims that line
and filters its sessions out; it doesn't open a separate tab. Each chip is also
a tiny status readout (`Claude 78%`) so the header doubles as a recap of the bar.

### Three distinct empty states

| Card | Empty message | Next step |
|---|---|---|
| Plan remaining | "Waiting for first poll" | Wait (passive) |
| Sessions | "No sessions yet" | Use Claude Code / Codex / Copilot |
| Next reset | "Sign in to see reset times" | Open Settings → Providers |

Each empty state names a different next step. Re-using one placeholder would
hide that distinction.

---

## 6. External dependencies

- **Pretendard** — Google Fonts (`https://fonts.googleapis.com/css2?family=Pretendard:wght@400;500;600;700`)
- **JetBrains Mono** — Google Fonts (`https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500`)

No JS libraries; both mockups are single-file HTML with inline `<style>`.

---

## 7. Icons (provider marks)

The three provider icons are simplified geometric interpretations of each
brand's mark — sized 14 × 14 px on the bar, redrawn inline as SVG so they take
status color via `currentColor`:

- **Claude** — 4-point burst with concave sides
- **Codex** — three overlapping ellipses (60° rotated)
- **Copilot** — rounded blob with two antennae and two eye dots

These are for prototyping. Before public release, double-check brand-guideline
licensing for each provider and swap to their official SVG/PNG marks
(open issue § 11 of the spec).
