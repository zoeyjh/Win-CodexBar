import { useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties } from "react";
import type { UsageProvider, UsageSnapshot } from "../types/usage";

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
}: {
  provider: UsageProvider;
  snapshot: UsageSnapshot | null;
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
    if (snapshot?.status === "auth_expired") {
      return "floatbar__pill floatbar__pill--dim";
    }

    if (snapshot?.status === "rate_limited") {
      return "floatbar__pill floatbar__pill--warn";
    }

    return "floatbar__pill";
  }, [snapshot?.status]);

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
        <div className="floatbar__tooltip" role="tooltip">
          <div className="floatbar__tooltip-title">{providerName}</div>
          {snapshot?.plan ? <div className="floatbar__tooltip-line">{snapshot.plan}</div> : null}
          <div className="floatbar__tooltip-line">{tooltipReset}</div>
        </div>
      ) : null}
    </div>
  );
}
