import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import StepHotkey from "../StepHotkey";

const mockInvoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockInvoke.mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
});

describe("StepHotkey", () => {
  it("default binding is fn", async () => {
    mockInvoke.mockResolvedValue(false);
    render(
      <StepHotkey
        onComplete={vi.fn()}
        binding={{ mode: "fn" }}
        setBinding={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("fn")).toBeTruthy();
    });
  });

  it("enables Continue when conflicts is false", async () => {
    mockInvoke.mockResolvedValue(false);
    render(
      <StepHotkey
        onComplete={vi.fn()}
        binding={{ mode: "fn" }}
        setBinding={vi.fn()}
      />
    );

    await waitFor(() => {
      const continueBtn = screen.getByText("Continue");
      expect(continueBtn.hasAttribute("disabled")).toBe(false);
    });
  });

  it("disables Continue when conflicts is true", async () => {
    mockInvoke.mockResolvedValue(true);
    render(
      <StepHotkey
        onComplete={vi.fn()}
        binding={{ mode: "fn" }}
        setBinding={vi.fn()}
      />
    );

    await waitFor(() => {
      const continueBtn = screen.getByText("Continue");
      expect(continueBtn.hasAttribute("disabled")).toBe(true);
    });
  });

  it("disables Continue when binding is null", () => {
    mockInvoke.mockResolvedValue(false);
    render(
      <StepHotkey onComplete={vi.fn()} binding={null} setBinding={vi.fn()} />
    );

    const continueBtn = screen.getByText("Continue");
    expect(continueBtn.hasAttribute("disabled")).toBe(true);
  });

  it("calls platform_register_hotkey on Continue click", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(false);
      return Promise.resolve(undefined);
    });
    const onComplete = vi.fn();

    render(
      <StepHotkey
        onComplete={onComplete}
        binding={{ mode: "fn" }}
        setBinding={vi.fn()}
      />
    );

    // Continue stays disabled until the async hotkey-conflict check resolves
    // (conflicts === false). Wait for it to be ENABLED, not merely present — else
    // under load the click lands on a still-disabled button and
    // platform_register_hotkey_cmd never fires (flaky failure at the gate).
    await waitFor(() => {
      expect(screen.getByText("Continue").hasAttribute("disabled")).toBe(false);
    });

    fireEvent.click(screen.getByText("Continue"));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("platform_register_hotkey_cmd", {
        binding: "fn",
      });
    });

    expect(onComplete).toHaveBeenCalledOnce();
  });
});
