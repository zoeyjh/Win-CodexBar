export type UsageProvider = "claude" | "codex" | "copilot";
export type UsageUnit = "token" | "request" | "percent";
export type UsageStatus =
  | "ok"
  | "stale"
  | "auth_expired"
  | "rate_limited"
  | "network"
  | "unknown";
export type UsageConfidence = "high" | "cached" | "inferred";

export interface UsageSnapshot {
  provider: UsageProvider;
  plan: string | null;
  unit: UsageUnit;
  used: number | null;
  limit: number | null;
  remaining_pct: number | null;
  reset_at: string | null;
  status: UsageStatus;
  last_success_at: string;
  confidence: UsageConfidence;
}

export const USAGE_PROVIDERS: UsageProvider[] = ["claude", "codex", "copilot"];
