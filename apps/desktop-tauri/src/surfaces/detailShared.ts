import type { UsageProvider, UsageSnapshot } from "../types/usage";

export const DETAIL_PROVIDER_META: Record<UsageProvider, { label: string; color: string }> = {
  claude: { label: "Claude", color: "#C97A3E" },
  codex: { label: "Codex", color: "#4A9EFF" },
  copilot: { label: "Copilot", color: "#7B61FF" },
};

export function formatProviderPercent(snapshot: UsageSnapshot | null): string {
  if (!snapshot || snapshot.remaining_pct === null) {
    return "--";
  }
  return `${Math.round(snapshot.remaining_pct)}%`;
}

export function formatDateTime(value: string | null): string {
  if (!value) {
    return "Unavailable";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "Unavailable";
  }
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
}

export function formatCountdown(resetAt: string | null): string {
  if (!resetAt) {
    return "--";
  }
  const remainingMs = new Date(resetAt).getTime() - Date.now();
  if (Number.isNaN(remainingMs)) {
    return "--";
  }
  const totalMinutes = Math.max(0, Math.ceil(remainingMs / 60_000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return `${hours}h ${minutes}m remaining`;
}

export function formatHistoryLabel(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}

export function formatSessionTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
    month: "short",
    day: "numeric",
  }).format(date);
}

export function formatTokens(tokens: number | null): string {
  if (tokens === null || !Number.isFinite(tokens)) {
    return "--";
  }
  return new Intl.NumberFormat().format(tokens);
}
