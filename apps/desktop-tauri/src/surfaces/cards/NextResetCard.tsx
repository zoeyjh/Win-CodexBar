import { openSettingsWindow } from "../../lib/tauri";
import type { UsageProvider, UsageSnapshot } from "../../types/usage";
import {
  DETAIL_PROVIDER_META,
  formatCountdown,
  formatDateTime,
} from "../detailShared";

export default function NextResetCard({
  activeProviders,
  snapshots,
}: {
  activeProviders: UsageProvider[];
  snapshots: Partial<Record<UsageProvider, UsageSnapshot | null>>;
}) {
  const providersWithReset = activeProviders.filter((provider) => snapshots[provider]?.reset_at);

  if (providersWithReset.length === 0) {
    return (
      <section className="detail-card detail-card--reset" aria-labelledby="detail-reset-heading">
        <div className="detail-card__header">
          <div>
            <h2 id="detail-reset-heading">Next Reset</h2>
            <p>Provider reset schedule</p>
          </div>
        </div>
        <div className="detail-card__empty detail-card__empty--stacked">
          <span>Sign in to see reset times</span>
          <button type="button" className="detail-card__button" onClick={() => openSettingsWindow("providers")}>
            Open Settings
          </button>
        </div>
      </section>
    );
  }

  return (
    <section className="detail-card detail-card--reset" aria-labelledby="detail-reset-heading">
      <div className="detail-card__header">
        <div>
          <h2 id="detail-reset-heading">Next Reset</h2>
          <p>Provider reset schedule</p>
        </div>
      </div>
      <div className="reset-card">
        {activeProviders.map((provider) => {
          const snapshot = snapshots[provider] ?? null;
          const remainingPct = snapshot?.remaining_pct ?? 0;
          const progressPct = Math.max(0, Math.min(100, 100 - remainingPct));
          return (
            <div key={provider} className="reset-card__item">
              <div className="reset-card__row">
                <span
                  className="reset-card__dot"
                  style={{ backgroundColor: DETAIL_PROVIDER_META[provider].color }}
                />
                <div className="reset-card__identity">
                  <strong>{DETAIL_PROVIDER_META[provider].label}</strong>
                  <span>{formatDateTime(snapshot?.reset_at ?? null)}</span>
                </div>
                <span className="reset-card__countdown">{formatCountdown(snapshot?.reset_at ?? null)}</span>
              </div>
              <div className="reset-card__timeline" aria-hidden>
                <span className="reset-card__timeline-fill" style={{ width: `${progressPct}%` }} />
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
