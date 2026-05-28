# R-1: Provider Data Fields

| Field | Claude | Codex (OpenAI) | Copilot (GitHub) |
|---|---|---|---|
| used | ✅ (utilization %) | ✅ (used_percent) | ✅ (100 - percent_remaining) |
| limit | ❌ (not exposed) | ✅ (limit_window_seconds, credits balance) | ✅ (entitlement) |
| remaining_pct | ✅ (computed from utilization) | ✅ (100 - used_percent) | ✅ (percent_remaining) |
| reset_at | ✅ (resets_at) | ✅ (reset_at) | ✅ (quota_reset_date) |
| plan | ✅ (rate_limit_tier) | ✅ (plan_type) | ✅ (copilot_plan) |
| sessions | ❌ | ❌ | ❌ |
| model_breakdown | Partial (named windows: fiveHour, sevenDay*) | Partial (rate windows) | Partial (quota_snapshots) |

## Notes
- Claude: OAuth path has named rate windows (fiveHour, sevenDay variants). Web API similar.
- Codex/OpenAI: rate_limit object with primary/secondary windows. Credits system separate.
- Copilot: quotas normalized from snapshot map (chat, completions, premium).
- status.rs only stores service health, NOT usage data. Usage lives in each provider module.

## Impact on UsageSnapshot
All 3 providers can populate: provider, remaining_pct, reset_at, plan, status.
`used` and `limit` available for Codex/Copilot but Claude only gives percentage.
→ UsageSnapshot.used/limit should be Optional as designed.
