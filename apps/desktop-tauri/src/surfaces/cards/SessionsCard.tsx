import { useEffect, useMemo, useState } from "react";
import { getProviderSessions } from "../../lib/tauri";
import type { SessionLogEntry, UsageProvider } from "../../types/usage";
import {
  DETAIL_PROVIDER_META,
  formatSessionTime,
  formatTokens,
} from "../detailShared";

const SESSION_PROVIDERS: UsageProvider[] = ["claude", "codex", "copilot"];

export default function SessionsCard({ activeProviders }: { activeProviders: UsageProvider[] }) {
  const [sessions, setSessions] = useState<Partial<Record<UsageProvider, SessionLogEntry[]>>>({});

  useEffect(() => {
    let cancelled = false;
    Promise.all(
      SESSION_PROVIDERS.map(async (provider) => [provider, await getProviderSessions(provider)] as const),
    )
      .then((entries) => {
        if (!cancelled) {
          setSessions(Object.fromEntries(entries));
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSessions({});
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const visibleGroups = useMemo(
    () => activeProviders.map((provider) => ({ provider, rows: sessions[provider] ?? [] })),
    [activeProviders, sessions],
  );

  return (
    <section className="detail-card detail-card--sessions" aria-labelledby="detail-sessions-heading">
      <div className="detail-card__header">
        <div>
          <h2 id="detail-sessions-heading">Sessions</h2>
          <p>Time, model, tokens, directory</p>
        </div>
      </div>
      <div className="sessions-card">
        <div className="sessions-card__table sessions-card__table--header" role="row">
          <span>Time</span>
          <span>Model</span>
          <span>Tokens</span>
          <span>Directory</span>
        </div>
        <div className="sessions-card__groups">
          {visibleGroups.map(({ provider, rows }) => (
            <section key={provider} className="sessions-card__group" aria-label={`${provider}-sessions`}>
              <div className="sessions-card__provider">{DETAIL_PROVIDER_META[provider].label}</div>
              {provider === "copilot" ? (
                <div className="detail-card__empty detail-card__empty--inline">
                  Session log not available for this provider
                </div>
              ) : provider === "codex" ? (
                <div className="detail-card__empty detail-card__empty--inline">
                  Codex session log coming soon
                </div>
              ) : rows.length === 0 ? (
                <div className="detail-card__empty detail-card__empty--inline">No sessions yet</div>
              ) : (
                rows.slice(-10).reverse().map((row, index) => (
                  <div key={`${provider}-${row.time}-${index}`} className="sessions-card__table" role="row">
                    <span>{formatSessionTime(row.time)}</span>
                    <span>{row.model ?? "Unknown"}</span>
                    <span>{formatTokens(row.tokens)}</span>
                    <span title={row.directory ?? ""}>{row.directory ?? "--"}</span>
                  </div>
                ))
              )}
            </section>
          ))}
        </div>
      </div>
    </section>
  );
}
