# R-5: Chart Library

## Finding
No external chart library in package.json.

## Existing Custom Chart Components
- `src/components/charts/BarChart.tsx` (173 lines) — custom SVG bar chart
- `src/components/charts/LineChart.tsx` (168 lines) — custom SVG line chart
- `src/surfaces/settings/providers/sections/charts/UsageBreakdownChart.tsx` (176 lines)
- Used by ChartsSection.tsx, CostHistoryChart.tsx, CreditsHistoryChart.tsx

## CSS Approach
Plain CSS + CSS custom properties (no Tailwind, no styled-components).
Theme tokens in `src/styles.css`.

## React Version
18.3.1

## Decision
**Use existing custom SVG LineChart component** for Card 1 (Plan Remaining).
- Already handles time-series data
- No new dependency needed
- Consistent with codebase style
- May need minor extension for dashed projection line
