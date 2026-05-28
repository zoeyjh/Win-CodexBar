import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ProviderUsageSnapshot } from "../types/bridge";

const useProvidersMocks = vi.hoisted(() => ({
  useProviders: vi.fn(),
}));

vi.mock("../hooks/useProviders", () => useProvidersMocks);

import DetailView from "./DetailView";

function buildSnapshot(
  overrides: Partial<ProviderUsageSnapshot>,
): ProviderUsageSnapshot {
  return {
    providerId: "claude",
    displayName: "Claude",
    primary: {
      usedPercent: 42,
      remainingPercent: 58,
      windowMinutes: 300,
      resetsAt: "2026-05-28T10:30:00Z",
      resetDescription: null,
      isExhausted: false,
      reservePercent: null,
      reserveDescription: null,
    },
    primaryLabel: "Session",
    secondary: null,
    secondaryLabel: undefined,
    modelSpecific: null,
    tertiary: null,
    extraRateWindows: [],
    cost: null,
    planName: null,
    accountEmail: null,
    sourceLabel: "CLI",
    updatedAt: "2026-05-28T09:00:00Z",
    error: null,
    pace: null,
    accountOrganization: null,
    trayStatusLabel: null,
    fetchDurationMs: null,
    ...overrides,
  };
}

describe("DetailView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useProvidersMocks.useProviders.mockReturnValue({
      providers: [],
      isRefreshing: false,
      refresh: vi.fn(),
      lastRefresh: null,
      hasCachedData: false,
    });
  });

  it("renders grouped provider windows from live provider snapshots", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-28T09:00:00Z"));

    useProvidersMocks.useProviders.mockReturnValue({
      providers: [
        buildSnapshot({
          planName: "Pro",
          secondary: {
            usedPercent: 68,
            remainingPercent: 32,
            windowMinutes: 10080,
            resetsAt: "2026-05-29T09:00:00Z",
            resetDescription: null,
            isExhausted: false,
            reservePercent: null,
            reserveDescription: null,
          },
          secondaryLabel: "Weekly",
          extraRateWindows: [
            {
              id: "extra",
              title: "Bonus",
              window: {
                usedPercent: 15,
                remainingPercent: 85,
                windowMinutes: 1440,
                resetsAt: "2026-05-28T09:20:00Z",
                resetDescription: null,
                isExhausted: false,
                reservePercent: null,
                reserveDescription: null,
              },
            },
          ],
        }),
      ],
      isRefreshing: false,
      refresh: vi.fn(),
      lastRefresh: null,
      hasCachedData: true,
    });

    try {
      render(<DetailView state={{} as never} />);

      expect(screen.getByText("Next Reset")).toBeInTheDocument();
      expect(screen.getByText("Claude")).toBeInTheDocument();
      expect(screen.getByText("Pro")).toBeInTheDocument();
      expect(screen.getByText(/42% used/i)).toBeInTheDocument();
      expect(screen.getByText(/68% used/i)).toBeInTheDocument();
      expect(screen.getByText(/15% used/i)).toBeInTheDocument();
      expect(screen.getByText("Session")).toBeInTheDocument();
      expect(screen.getByText("Weekly")).toBeInTheDocument();
      expect(screen.getByText("Bonus")).toBeInTheDocument();
      expect(screen.getByText(/1h 30m/i)).toBeInTheDocument();
      expect(screen.getByText(/20m/i)).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("filters the grouped reset view to a single provider when providerId is set", () => {
    useProvidersMocks.useProviders.mockReturnValue({
      providers: [
        buildSnapshot({ providerId: "claude", displayName: "Claude" }),
        buildSnapshot({ providerId: "codex", displayName: "Codex" }),
      ],
      isRefreshing: false,
      refresh: vi.fn(),
      lastRefresh: null,
      hasCachedData: true,
    });

    render(<DetailView state={{} as never} providerId="codex" />);

    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.queryByText("Claude")).not.toBeInTheDocument();
  });
});
