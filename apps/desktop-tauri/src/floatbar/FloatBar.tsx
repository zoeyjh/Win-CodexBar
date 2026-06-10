import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { emitCachedUsageUpdates, getSettingsSnapshot } from "../lib/tauri";
import type { BootstrapState } from "../types/bridge";
import { USAGE_PROVIDERS, type UsageProvider, type UsageSnapshot } from "../types/usage";
import ProviderPill from "./ProviderPill";
import { useDragOrClick } from "./useDragOrClick";
import { FLOAT_BAR_CONFIG_CHANGED_EVENT, setFloatBarHitRect } from "./api";
import "./floatbar.css";

const TOOLTIP_OPEN_DELAY_MS = 250;

const EMPTY_SNAPSHOTS: Record<UsageProvider, UsageSnapshot | null> = {
  claude: null,
  codex: null,
  copilot: null,
};

function formatProviderName(provider: UsageProvider): string {
  return provider.charAt(0).toUpperCase() + provider.slice(1);
}

function formatRemaining(totalMinutes: number): string {
  const days = Math.floor(totalMinutes / 1440);
  const hours = Math.floor((totalMinutes % 1440) / 60);
  const minutes = totalMinutes % 60;
  if (days > 0) {
    return `${days}d ${hours}h left`;
  }
  if (hours > 0) {
    return `${hours}h ${minutes}m left`;
  }
  return `${minutes}m left`;
}

/**
 * Reset display split into a primary line (countdown or clock time) and an
 * optional secondary line (the absolute-mode remaining suffix). The two are
 * rendered on separate lines so they don't overflow the narrow tooltip.
 */
interface ResetDisplay {
  time: string;
  remaining: string | null;
}

function formatReset(resetAt: string | null, relative: boolean): ResetDisplay {
  if (!resetAt) {
    return { time: "Reset unavailable", remaining: null };
  }

  const resetMs = new Date(resetAt).getTime();
  if (Number.isNaN(resetMs)) {
    return { time: "Reset unavailable", remaining: null };
  }

  if (relative) {
    const totalMinutes = Math.max(0, Math.ceil((resetMs - Date.now()) / 60_000));
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    return { time: `Resets in ${hours}h ${minutes}m`, remaining: null };
  }

  // Absolute mode: show the clock time (date omitted when the reset falls on
  // today) with the remaining countdown on its own line, e.g.
  //   3:00 PM
  //   (2h 33m left)
  const now = new Date();
  const resetDate = new Date(resetMs);
  const isToday =
    resetDate.getFullYear() === now.getFullYear() &&
    resetDate.getMonth() === now.getMonth() &&
    resetDate.getDate() === now.getDate();

  let clock: string;
  try {
    clock = new Intl.DateTimeFormat(undefined, {
      ...(isToday ? {} : { month: "short", day: "numeric" }),
      hour: "numeric",
      minute: "2-digit",
    }).format(resetDate);
  } catch {
    return { time: "Reset unavailable", remaining: null };
  }

  const totalMinutes = Math.max(0, Math.ceil((resetMs - now.getTime()) / 60_000));
  return {
    time: clock,
    remaining: totalMinutes > 0 ? `(${formatRemaining(totalMinutes)})` : null,
  };
}

export default function FloatBar({ state }: { state: BootstrapState }) {
  const [snapshots, setSnapshots] = useState<Record<UsageProvider, UsageSnapshot | null>>(
    EMPTY_SNAPSHOTS,
  );
  const dragOrClick = useDragOrClick();
  const [hoveredProvider, setHoveredProvider] = useState<UsageProvider | null>(null);
  const [relativeReset, setRelativeReset] = useState<boolean>(
    () => state?.settings?.resetTimeRelative ?? true,
  );
  const openTimerRef = useRef<number | null>(null);
  const barRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    document.body.classList.add("floatbar-window");
    document.documentElement.classList.add("floatbar-window-root");
    return () => {
      document.body.classList.remove("floatbar-window");
      document.documentElement.classList.remove("floatbar-window-root");
    };
  }, []);

  useEffect(() => {
    const node = barRef.current;
    if (!node) {
      return;
    }
    const report = () => {
      const r = node.getBoundingClientRect();
      void setFloatBarHitRect({ x: r.x, y: r.y, w: r.width, h: r.height }).catch(() => {});
    };
    report();
    const observer = new ResizeObserver(report);
    observer.observe(node);
    return () => observer.disconnect();
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

  // The float-bar window bootstraps its settings once, so re-read the reset
  // format whenever the shell signals a settings change. Without this the
  // "Relative Reset time" toggle would only take effect after the bar is
  // recreated.
  useEffect(() => {
    const unlisten = listen(FLOAT_BAR_CONFIG_CHANGED_EVENT, () => {
      void getSettingsSnapshot()
        .then((settings) => setRelativeReset(settings.resetTimeRelative))
        .catch(() => {});
    });
    return () => {
      void unlisten.then((stopListening) => stopListening()).catch(() => {});
    };
  }, []);

  useEffect(() => {
    return () => {
      if (openTimerRef.current !== null) {
        window.clearTimeout(openTimerRef.current);
      }
    };
  }, []);

  const handleHoverStart = useCallback((provider: UsageProvider) => {
    if (openTimerRef.current !== null) {
      window.clearTimeout(openTimerRef.current);
    }

    // If the tooltip is already open, swap content immediately so moving
    // between pills doesn't flash. Otherwise wait the open delay.
    setHoveredProvider((current) => (current !== null ? provider : current));
    openTimerRef.current = window.setTimeout(() => {
      setHoveredProvider(provider);
    }, TOOLTIP_OPEN_DELAY_MS);
  }, []);

  const handleHoverEnd = useCallback(() => {
    if (openTimerRef.current !== null) {
      window.clearTimeout(openTimerRef.current);
      openTimerRef.current = null;
    }
    setHoveredProvider(null);
  }, []);

  const hoveredSnapshot = hoveredProvider ? snapshots[hoveredProvider] : null;
  const resetInfo = formatReset(hoveredSnapshot?.reset_at ?? null, relativeReset);

  return (
    <div
      ref={barRef}
      className="floatbar"
      onContextMenu={(event) => event.preventDefault()}
      {...dragOrClick}
    >
      {USAGE_PROVIDERS.map((provider) => (
        <ProviderPill
          key={provider}
          provider={provider}
          snapshot={snapshots[provider]}
          onHoverStart={handleHoverStart}
          onHoverEnd={handleHoverEnd}
        />
      ))}
      {hoveredProvider !== null ? (
        <div className="floatbar__tooltip" role="tooltip">
          <div className="floatbar__tooltip-title">{formatProviderName(hoveredProvider)}</div>
          {hoveredSnapshot?.plan ? (
            <div className="floatbar__tooltip-line">{hoveredSnapshot.plan}</div>
          ) : null}
          <div className="floatbar__tooltip-line">{resetInfo.time}</div>
          {resetInfo.remaining ? (
            <div className="floatbar__tooltip-line">{resetInfo.remaining}</div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
