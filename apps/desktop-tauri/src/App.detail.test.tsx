import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { buildBundle } from "./test/localeHarness";

const tauriMocks = vi.hoisted(() => ({
  getBootstrapState: vi.fn(),
  getSettingsSnapshot: vi.fn(),
  checkForUpdates: vi.fn(),
  setSurfaceMode: vi.fn(),
  getLocaleStrings: vi.fn(),
  setUiLanguage: vi.fn(),
  getCachedProviders: vi.fn(),
  refreshProviders: vi.fn(),
  refreshProvidersIfStale: vi.fn(),
  emitCachedUsageUpdates: vi.fn(),
  getUpdateState: vi.fn(),
  openSettingsWindow: vi.fn(),
  quitApp: vi.fn(),
  getProviderChartData: vi.fn(),
  getUsageHistory: vi.fn(),
  getProviderSessions: vi.fn(),
  getApiKeys: vi.fn(),
  getManualCookies: vi.fn(),
  getTokenAccountProviders: vi.fn(),
  getTokenAccounts: vi.fn(),
}));

const eventMocks = vi.hoisted(() => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const webviewMocks = vi.hoisted(() => ({
  getCurrentWebviewWindow: vi.fn(() => ({ label: "main" })),
}));

const windowMocks = vi.hoisted(() => ({
  getCurrentWindow: vi.fn(() => ({
    setSize: vi.fn().mockResolvedValue(undefined),
    setPosition: vi.fn().mockResolvedValue(undefined),
  })),
  LogicalSize: vi.fn((width: number, height: number) => ({ width, height })),
  LogicalPosition: vi.fn((x: number, y: number) => ({ x, y })),
}));

vi.mock("./lib/tauri", () => tauriMocks);
vi.mock("@tauri-apps/api/event", () => eventMocks);
vi.mock("@tauri-apps/api/webviewWindow", () => webviewMocks);
vi.mock("@tauri-apps/api/window", () => windowMocks);
vi.mock("./hooks/useSurfaceSnapshot", () => ({
  useSurfaceSnapshot: () => ({ mode: "popOut", target: { kind: "dashboard" } }),
}));

import App from "./App";

describe("App detail surface", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    tauriMocks.getBootstrapState.mockResolvedValue({
      contractVersion: "v1",
      surfaceModes: [],
      commands: [],
      events: [],
      providers: [],
      settings: {
        theme: "dark",
      },
    });
    tauriMocks.getSettingsSnapshot.mockResolvedValue({ theme: "dark" });
    tauriMocks.checkForUpdates.mockResolvedValue(undefined);
    tauriMocks.setSurfaceMode.mockResolvedValue(undefined);
    tauriMocks.getLocaleStrings.mockResolvedValue(buildBundle());
    tauriMocks.getCachedProviders.mockResolvedValue([
      {
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
        secondaryLabel: null,
        modelSpecific: null,
        tertiary: null,
        extraRateWindows: [],
        cost: null,
        planName: "Pro",
        accountEmail: null,
        sourceLabel: "CLI",
        updatedAt: "2026-05-28T09:00:00Z",
        error: null,
        pace: null,
        accountOrganization: null,
        trayStatusLabel: "42%",
        fetchDurationMs: null,
      },
    ]);
    tauriMocks.refreshProviders.mockResolvedValue(undefined);
    tauriMocks.refreshProvidersIfStale.mockResolvedValue(undefined);
    tauriMocks.emitCachedUsageUpdates.mockResolvedValue(undefined);
    tauriMocks.getUpdateState.mockResolvedValue({
      status: "idle",
      version: null,
      error: null,
      progress: null,
      releaseUrl: null,
      canDownload: false,
      canApply: false,
      lastCheckedAt: null,
    });
    tauriMocks.getApiKeys.mockResolvedValue([]);
    tauriMocks.getManualCookies.mockResolvedValue([]);
    tauriMocks.getTokenAccountProviders.mockResolvedValue([]);
    tauriMocks.getTokenAccounts.mockResolvedValue({ accounts: [] });
    tauriMocks.getProviderChartData.mockResolvedValue({
      providerId: "claude",
      costHistory: [],
      creditsHistory: [],
      usageBreakdown: [],
      localUsage: null,
    });
    tauriMocks.getUsageHistory.mockImplementation(async (provider: string) => [{
      provider,
      timestamp: "2026-05-28T10:00:00Z",
      remaining_pct: 88,
    }]);
    tauriMocks.getProviderSessions.mockResolvedValue([
      {
        provider: "claude",
        time: "2026-05-28T09:15:00Z",
        model: "claude-3.7-sonnet",
        tokens: 1200,
        directory: "E:/repo",
      },
    ]);
  });

  it("renders the grouped Next Reset detail view when the shell is in popOut mode", async () => {
    render(<App />);

    // Wait for the live provider snapshot to land — "Next Reset" is the card
    // header and renders even in the empty state, so gate on actual content.
    await waitFor(() => {
      expect(screen.getByText("Claude")).toBeInTheDocument();
    });
    expect(screen.getByText("Next Reset")).toBeInTheDocument();
    expect(screen.getByText("Pro")).toBeInTheDocument();
    expect(screen.getByText("Session")).toBeInTheDocument();
  });

  it("opens the detail view on first run when no provider credentials exist", async () => {
    render(<App />);

    await waitFor(() => {
      expect(tauriMocks.setSurfaceMode).toHaveBeenCalledWith("popOut", { kind: "dashboard" });
    });
  });
});
