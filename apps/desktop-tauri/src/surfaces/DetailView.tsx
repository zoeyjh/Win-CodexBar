import { useMemo } from "react";
import { useProviders } from "../hooks/useProviders";
import { useSettings } from "../hooks/useSettings";
import type { BootstrapState } from "../types/bridge";
import { USAGE_PROVIDERS } from "../types/usage";
import NextResetCard from "./cards/NextResetCard";
import "./detail.css";

export default function DetailView({
  state,
  providerId,
}: {
  state: BootstrapState;
  providerId?: string;
}) {
  const { providers } = useProviders();
  const { settings } = useSettings(state.settings);

  const filteredProviders = useMemo(() => {
    if (providerId && USAGE_PROVIDERS.includes(providerId as any)) {
      return providers.filter((p) => p.providerId === providerId);
    }
    return providers;
  }, [providers, providerId]);

  return (
    <main className="detail-view">
      <section className="detail-view__grid">
        <NextResetCard providers={filteredProviders} relative={settings.resetTimeRelative} />
      </section>
    </main>
  );
}
