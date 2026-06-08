import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { emitCachedUsageUpdates } from "../lib/tauri";
import type { BootstrapState } from "../types/bridge";
import { USAGE_PROVIDERS, type UsageProvider, type UsageSnapshot } from "../types/usage";
import ProviderPill from "./ProviderPill";
import { useDragOrClick } from "./useDragOrClick";
import { setFloatBarHitRect } from "./api";
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

export default function FloatBar({ state: _state }: { state: BootstrapState }) {
  const [snapshots, setSnapshots] = useState<Record<UsageProvider, UsageSnapshot | null>>(
    EMPTY_SNAPSHOTS,
  );
  const dragOrClick = useDragOrClick();
  const [hoveredProvider, setHoveredProvider] = useState<UsageProvider | null>(null);
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
          <div className="floatbar__tooltip-line">
            {formatResetTime(hoveredSnapshot?.reset_at ?? null)}
          </div>
        </div>
      ) : null}
    </div>
  );
}
