import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { emitCachedUsageUpdates } from "../lib/tauri";
import type { BootstrapState } from "../types/bridge";
import { USAGE_PROVIDERS, type UsageProvider, type UsageSnapshot } from "../types/usage";
import ProviderPill from "./ProviderPill";
import { useDragOrClick } from "./useDragOrClick";
import "./floatbar.css";

const EMPTY_SNAPSHOTS: Record<UsageProvider, UsageSnapshot | null> = {
  claude: null,
  codex: null,
  copilot: null,
};

export default function FloatBar({ state: _state }: { state: BootstrapState }) {
  const [snapshots, setSnapshots] = useState<Record<UsageProvider, UsageSnapshot | null>>(
    EMPTY_SNAPSHOTS,
  );
  const dragOrClick = useDragOrClick();

  useEffect(() => {
    document.body.classList.add("floatbar-window");
    return () => {
      document.body.classList.remove("floatbar-window");
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
        <ProviderPill key={provider} provider={provider} snapshot={snapshots[provider]} />
      ))}
    </div>
  );
}
