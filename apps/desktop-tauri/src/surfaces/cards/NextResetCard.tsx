import type { ProviderUsageSnapshot } from "../../types/bridge";
import { openSettingsWindow } from "../../lib/tauri";
import {
  PROVIDER_COLORS,
  collectRateWindows,
  formatWindowCountdown,
} from "../detailShared";

export default function NextResetCard({
  providers,
  relative,
}: {
  providers: ProviderUsageSnapshot[];
  relative: boolean;
}) {
  const activeProviders = providers.filter((p) => !p.error);

  if (activeProviders.length === 0) {
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
      <div className="reset-groups">
        {activeProviders.map((provider) => {
          const color = PROVIDER_COLORS[provider.providerId] ?? "#888";
          const windows = collectRateWindows(provider);
          return (
            <div key={provider.providerId} className="reset-group">
              <div className="reset-group__header">
                <span className="reset-group__dot" style={{ backgroundColor: color }} />
                <strong className="reset-group__name">{provider.displayName}</strong>
                {provider.planName && (
                  <span className="reset-group__plan">{provider.planName}</span>
                )}
              </div>
              <div className="reset-group__windows">
                {windows.map((entry) => (
                  <div key={entry.label} className="reset-window">
                    <div className="reset-window__row">
                      <span className="reset-window__label">{entry.label}</span>
                      <span className="reset-window__info">
                        {Math.round(entry.window.usedPercent)}% used · {formatWindowCountdown(entry.window.resetsAt, relative)}
                      </span>
                    </div>
                    <div className="reset-window__bar">
                      <span
                        className="reset-window__fill"
                        style={{
                          width: `${Math.min(100, entry.window.usedPercent)}%`,
                          backgroundColor: color,
                        }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
