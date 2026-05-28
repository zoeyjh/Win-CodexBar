import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { BootstrapState } from "../types/bridge";
import { USAGE_PROVIDERS, type UsageProvider, type UsageSnapshot } from "../types/usage";
import { DETAIL_PROVIDER_META, formatProviderPercent } from "./detailShared";
import PlanRemainingCard from "./cards/PlanRemainingCard";
import SessionsCard from "./cards/SessionsCard";
import NextResetCard from "./cards/NextResetCard";
import "./detail.css";

const EMPTY_SNAPSHOTS: Partial<Record<UsageProvider, UsageSnapshot | null>> = {
  claude: null,
  codex: null,
  copilot: null,
};

function initialProviders(providerId?: string): UsageProvider[] {
  return USAGE_PROVIDERS.includes(providerId as UsageProvider)
    ? [providerId as UsageProvider]
    : [...USAGE_PROVIDERS];
}

export default function DetailView({
  state: _state,
  providerId,
}: {
  state: BootstrapState;
  providerId?: string;
}) {
  const [snapshots, setSnapshots] = useState<Partial<Record<UsageProvider, UsageSnapshot | null>>>(
    EMPTY_SNAPSHOTS,
  );
  const [activeProviders, setActiveProviders] = useState<UsageProvider[]>(() => initialProviders(providerId));

  useEffect(() => {
    setActiveProviders(initialProviders(providerId));
  }, [providerId]);

  useEffect(() => {
    const unlisten = listen<UsageSnapshot>("usage:update", (event) => {
      setSnapshots((current) => ({
        ...current,
        [event.payload.provider]: event.payload,
      }));
    });

    return () => {
      void unlisten.then((stopListening) => stopListening()).catch(() => {});
    };
  }, []);

  const orderedActiveProviders = useMemo(
    () => USAGE_PROVIDERS.filter((provider) => activeProviders.includes(provider)),
    [activeProviders],
  );

  const toggleProvider = (provider: UsageProvider) => {
    setActiveProviders((current) => {
      if (current.includes(provider)) {
        return current.length === 1 ? current : current.filter((item) => item !== provider);
      }
      return [...current, provider];
    });
  };

  return (
    <main className="detail-view">
      <header className="detail-view__header">
        <div>
          <div className="detail-view__eyebrow">Detail View</div>
          <h1>codexbar-zoey</h1>
        </div>
        <div className="detail-view__toggles" role="group" aria-label="Providers">
          {USAGE_PROVIDERS.map((provider) => {
            const active = orderedActiveProviders.includes(provider);
            return (
              <button
                key={provider}
                type="button"
                className={`detail-view__toggle${active ? " is-active" : ""}`}
                aria-pressed={active}
                onClick={() => toggleProvider(provider)}
              >
                <span
                  className="detail-view__toggle-dot"
                  style={{ backgroundColor: DETAIL_PROVIDER_META[provider].color }}
                />
                <span>{DETAIL_PROVIDER_META[provider].label}</span>
                <strong>{formatProviderPercent(snapshots[provider] ?? null)}</strong>
              </button>
            );
          })}
        </div>
      </header>

      <section className="detail-view__grid">
        <PlanRemainingCard activeProviders={orderedActiveProviders} />
        <SessionsCard activeProviders={orderedActiveProviders} />
        <NextResetCard activeProviders={orderedActiveProviders} snapshots={snapshots} />
      </section>
    </main>
  );
}
