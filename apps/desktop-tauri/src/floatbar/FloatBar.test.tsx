import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BootstrapState } from "../types/bridge";
import type { UsageSnapshot } from "../types/usage";

const eventListeners = new Map<string, (event: { payload: unknown }) => void>();

const eventMocks = vi.hoisted(() => ({
  listen: vi.fn((event: string, handler: (event: { payload: unknown }) => void) => {
    eventListeners.set(event, handler);
    return Promise.resolve(() => {
      eventListeners.delete(event);
    });
  }),
}));

const coreMocks = vi.hoisted(() => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

const windowInstance = {
  startDragging: vi.fn().mockResolvedValue(undefined),
};

const windowMocks = vi.hoisted(() => ({
  getCurrentWindow: vi.fn(() => windowInstance),
}));

vi.mock("@tauri-apps/api/event", () => eventMocks);
vi.mock("@tauri-apps/api/core", () => coreMocks);
vi.mock("@tauri-apps/api/window", () => windowMocks);

import FloatBar from "./FloatBar";

function bootstrap(): BootstrapState {
  return {} as BootstrapState;
}

function emitUsage(snapshot: UsageSnapshot) {
  const handler = eventListeners.get("usage:update");
  if (!handler) {
    throw new Error("usage:update listener missing");
  }

  act(() => {
    handler({ payload: snapshot });
  });
}

function usageSnapshot(
  provider: UsageSnapshot["provider"],
  overrides: Partial<UsageSnapshot> = {},
): UsageSnapshot {
  return {
    provider,
    plan: null,
    unit: "percent",
    used: null,
    limit: null,
    remaining_pct: 72,
    reset_at: "2026-06-01T10:30:00.000Z",
    status: "ok",
    last_success_at: "2026-06-01T08:00:00.000Z",
    confidence: "high",
    ...overrides,
  };
}

describe("FloatBar", () => {
  beforeEach(() => {
    eventListeners.clear();
    vi.clearAllMocks();
  });

  it("renders three provider pills and applies usage:update payloads", () => {
    const { container } = render(<FloatBar state={bootstrap()} />);

    expect(eventMocks.listen).toHaveBeenCalledWith("usage:update", expect.any(Function));
    expect(container.querySelectorAll(".floatbar__pill")).toHaveLength(3);
    expect(screen.getAllByText("--")).toHaveLength(3);

    emitUsage(usageSnapshot("claude", { remaining_pct: 81 }));

    expect(screen.getByTestId("provider-pill-claude")).toHaveTextContent("81%");
  });

  it("shows the delayed tooltip with provider, plan, and reset countdown", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-01T10:15:00.000Z"));

    try {
      render(<FloatBar state={bootstrap()} />);
      emitUsage(
        usageSnapshot("codex", {
          plan: "Pro",
          remaining_pct: 64,
          reset_at: "2026-06-01T11:45:00.000Z",
        }),
      );

      const pill = screen.getByTestId("provider-pill-codex");
      fireEvent.mouseEnter(pill);

      act(() => {
        vi.advanceTimersByTime(249);
      });
      expect(screen.queryByText("Codex")).toBeNull();

      act(() => {
        vi.advanceTimersByTime(1);
      });
      expect(screen.getByText("Codex")).toBeInTheDocument();
      expect(screen.getByText("Pro")).toBeInTheDocument();
      expect(screen.getByText("Resets in 1h 30m")).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("uses dim and warning states for auth-expired and rate-limited providers", () => {
    render(<FloatBar state={bootstrap()} />);

    emitUsage(usageSnapshot("claude", { status: "auth_expired", remaining_pct: null }));
    emitUsage(usageSnapshot("copilot", { status: "rate_limited", remaining_pct: 19 }));

    expect(screen.getByTestId("provider-pill-claude")).toHaveClass("floatbar__pill--dim");
    expect(screen.getByTestId("provider-pill-copilot")).toHaveClass("floatbar__pill--warn");
  });

  it("toggles detail on click and starts dragging after crossing the movement threshold", async () => {
    const { container } = render(<FloatBar state={bootstrap()} />);
    const bar = container.querySelector(".floatbar");
    expect(bar).not.toBeNull();

    fireEvent.mouseDown(bar!, { clientX: 10, clientY: 10, button: 0 });
    fireEvent.mouseUp(bar!, { clientX: 12, clientY: 12, button: 0 });
    expect(coreMocks.invoke).toHaveBeenCalledWith("toggle_detail");

    fireEvent.mouseDown(bar!, { clientX: 10, clientY: 10, button: 0 });
    fireEvent.mouseMove(bar!, { clientX: 20, clientY: 10, button: 0 });
    expect(windowInstance.startDragging).toHaveBeenCalledTimes(1);

    fireEvent.mouseUp(bar!, { clientX: 20, clientY: 10, button: 0 });
    await Promise.resolve();
    expect(coreMocks.invoke).toHaveBeenCalledTimes(1);
  });

  it("prevents the default context menu", () => {
    const { container } = render(<FloatBar state={bootstrap()} />);
    const bar = container.querySelector(".floatbar");
    expect(bar).not.toBeNull();

    const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
    const notCancelled = bar!.dispatchEvent(event);

    expect(notCancelled).toBe(false);
    expect(event.defaultPrevented).toBe(true);
  });
});
