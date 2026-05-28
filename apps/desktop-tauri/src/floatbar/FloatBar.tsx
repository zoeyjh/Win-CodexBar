import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
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
  const [direction, setDirection] = useState<TooltipDirection>("above");

  useEffect(() => {
    let cancelled = false;

    async function detectDirection() {
      try {
        const win = getCurrentWindow();
        const position = await win.outerPosition();
        const monitor = await currentMonitor();
        if (cancelled || !monitor) return;

        const monitorY = monitor.position.y;
        const halfHeight = monitor.size.height / 2;
        const relativeY = position.y - monitorY;

        setDirection(relativeY > halfHeight ? "above" : "below");
      } catch {
        // Fallback to above (most users place bar at bottom)
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
