import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { UsageSnapshot } from "../types/usage";

const listeners = new Map<string, (event: { payload: UsageSnapshot }) => void>();

const eventMocks = vi.hoisted(() => ({
  listen: vi.fn((event: string, handler: (event: { payload: UsageSnapshot }) => void) => {
    listeners.set(event, handler);
    return Promise.resolve(() => listeners.delete(event));
  }),
}));

const tauriMocks = vi.hoisted(() => ({
  getUsageHistory: vi.fn(),
  getProviderSessions: vi.fn(),
  openSettingsWindow: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => eventMocks);
vi.mock("../lib/tauri", () => tauriMocks);

import DetailView from "./DetailView";

function emitUsage(snapshot: UsageSnapshot) {
  const handler = listeners.get("usage:update");
  if (!handler) {
    throw new Error("usage:update listener missing");
  }
  act(() => {
    handler({ payload: snapshot });
  });
}

describe("DetailView", () => {
  beforeEach(() => {
    listeners.clear();
    vi.clearAllMocks();
    tauriMocks.getUsageHistory.mockResolvedValue([
      {
        provider: "claude",
        timestamp: "2026-05-28T10:00:00Z",
        remaining_pct: 90,
      },
    ]);
    tauriMocks.getProviderSessions.mockResolvedValue([]);
  });

  it("renders cards and keeps at least one provider toggle active", async () => {
    await act(async () => {
      render(<DetailView state={{} as never} />);
      await Promise.resolve();
    });

    await waitFor(() => {
      expect(screen.getByText("Plan Remaining")).toBeInTheDocument();
    });
    expect(screen.getByText("Sessions")).toBeInTheDocument();
    expect(screen.getByText("Next Reset")).toBeInTheDocument();

    const claude = screen.getByRole("button", { name: /Claude/i });
    const codex = screen.getByRole("button", { name: /Codex/i });
    const copilot = screen.getByRole("button", { name: /Copilot/i });

    fireEvent.click(codex);
    fireEvent.click(copilot);
    fireEvent.click(claude);

    expect(claude).toHaveAttribute("aria-pressed", "true");
  });

  it("applies usage:update payloads to the toggle summary and reset card", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-28T09:00:00Z"));

    try {
      await act(async () => {
        render(<DetailView state={{} as never} />);
        await Promise.resolve();
      });
      emitUsage({
        provider: "claude",
        plan: "Pro",
        unit: "percent",
        used: null,
        limit: null,
        remaining_pct: 81,
        reset_at: "2026-05-28T12:30:00Z",
        status: "ok",
        last_success_at: "2026-05-28T09:00:00Z",
        confidence: "high",
      });

      expect(screen.getByRole("button", { name: /Claude 81%/i })).toBeInTheDocument();
      expect(screen.getByText("3h 30m remaining")).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });
});
