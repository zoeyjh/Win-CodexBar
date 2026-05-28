import { useEffect, useMemo, useState } from "react";
import { LineChart, type LineChartPoint } from "../../components/charts/LineChart";
import { getUsageHistory } from "../../lib/tauri";
import type { UsageHistoryPoint, UsageProvider } from "../../types/usage";
import { DETAIL_PROVIDER_META, formatHistoryLabel } from "../detailShared";

const HISTORY_PROVIDERS: UsageProvider[] = ["claude", "codex", "copilot"];

function normalizeSeries(
  history: Partial<Record<UsageProvider, UsageHistoryPoint[]>>,
  activeProviders: UsageProvider[],
): Record<UsageProvider, LineChartPoint[]> {
  const timestamps = Array.from(new Set(activeProviders.flatMap((provider) =>
    (history[provider] ?? []).map((point) => point.timestamp),
  ))).sort();
  const visibleTimeline = timestamps.slice(-24);

  return activeProviders.reduce<Record<UsageProvider, LineChartPoint[]>>((result, provider) => {
    const points = [...(history[provider] ?? [])].sort((left, right) =>
      left.timestamp.localeCompare(right.timestamp),
    );
    let lastKnown = points[0]?.remaining_pct ?? 100;
    let cursor = 0;
    result[provider] = visibleTimeline.map((timestamp) => {
      while (cursor < points.length && points[cursor].timestamp <= timestamp) {
        lastKnown = points[cursor].remaining_pct;
        cursor += 1;
      }
      return {
        label: formatHistoryLabel(timestamp),
        value: lastKnown,
      };
    });
    return result;
  }, {
    claude: [],
    codex: [],
    copilot: [],
  });
}

export default function PlanRemainingCard({ activeProviders }: { activeProviders: UsageProvider[] }) {
  const [history, setHistory] = useState<Partial<Record<UsageProvider, UsageHistoryPoint[]>>>({});

  useEffect(() => {
    let cancelled = false;
    Promise.all(
      HISTORY_PROVIDERS.map(async (provider) => [provider, await getUsageHistory(provider)] as const),
    )
      .then((entries) => {
        if (cancelled) {
          return;
        }
        setHistory(Object.fromEntries(entries));
      })
      .catch(() => {
        if (!cancelled) {
          setHistory({});
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const normalized = useMemo(
    () => normalizeSeries(history, activeProviders),
    [activeProviders, history],
  );
  const hasData = activeProviders.some((provider) => (history[provider] ?? []).length > 0);

  return (
    <section className="detail-card detail-card--plan" aria-labelledby="detail-plan-heading">
      <div className="detail-card__header">
        <div>
          <h2 id="detail-plan-heading">Plan Remaining</h2>
          <p>Remaining % over time · 0–100%</p>
        </div>
      </div>
      {!hasData ? (
        <div className="detail-card__empty">Waiting for first poll...</div>
      ) : (
        <div className="detail-line-chart">
          {activeProviders.map((provider, index) => (
            <div
              key={provider}
              className={`detail-line-chart__series${index < activeProviders.length - 1 ? " detail-line-chart__series--overlay" : ""}`}
            >
              <LineChart
                ariaLabel={`${DETAIL_PROVIDER_META[provider].label} plan remaining chart`}
                data={normalized[provider]}
                color={DETAIL_PROVIDER_META[provider].color}
                area={index === 0}
                animations={false}
                minValue={0}
                maxValue={100}
                showAxis={index === activeProviders.length - 1}
                interactive={index === activeProviders.length - 1}
                valueFormatter={(value) => `${Math.round(value)}%`}
                emptyMessage="Waiting for first poll..."
              />
            </div>
          ))}
          <div className="detail-card__legend">
            {activeProviders.map((provider) => (
              <span key={provider} className="detail-card__legend-item">
                <span
                  className="detail-card__legend-dot"
                  style={{ backgroundColor: DETAIL_PROVIDER_META[provider].color }}
                />
                {DETAIL_PROVIDER_META[provider].label}
              </span>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}
