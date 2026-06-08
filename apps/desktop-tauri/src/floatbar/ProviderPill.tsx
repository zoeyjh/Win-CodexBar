import { useMemo } from "react";
import type { CSSProperties } from "react";
import type { UsageProvider, UsageSnapshot } from "../types/usage";

const PROVIDER_COLORS: Record<UsageProvider, string> = {
  claude: "#C97A3E",
  codex: "#4A9EFF",
  copilot: "#7B61FF",
};

function formatPercent(snapshot: UsageSnapshot | null): string {
  if (!snapshot || snapshot.status === "auth_expired" || snapshot.remaining_pct === null) {
    return "--";
  }

  return `${Math.round(snapshot.remaining_pct)}%`;
}

export default function ProviderPill({
  provider,
  snapshot,
  onHoverStart,
  onHoverEnd,
}: {
  provider: UsageProvider;
  snapshot: UsageSnapshot | null;
  onHoverStart?: (provider: UsageProvider) => void;
  onHoverEnd?: () => void;
}) {
  const pillClassName = useMemo(() => {
    if (!snapshot || snapshot.status === "auth_expired") {
      return "floatbar__pill floatbar__pill--dim";
    }

    if (snapshot.status === "rate_limited") {
      return "floatbar__pill floatbar__pill--warn";
    }

    return "floatbar__pill";
  }, [snapshot]);

  const percentage = formatPercent(snapshot);

  return (
    <div
      className={pillClassName}
      data-provider={provider}
      data-testid={`provider-pill-${provider}`}
      onMouseEnter={() => onHoverStart?.(provider)}
      onMouseLeave={() => onHoverEnd?.()}
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
    </div>
  );
}
