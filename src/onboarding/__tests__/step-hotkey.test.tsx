import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import StepHotkey from "../StepHotkey";
import { act } from "react";

const mockInvoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "platform_register_hotkey_cmd") return Promise.resolve(null);
    return Promise.resolve(undefined);
  });
});

afterEach(() => {
  cleanup();
});

describe("StepHotkey", () => {
  it("default binding is fn", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(false);
      return Promise.resolve(undefined);
    });
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
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(false);
      return Promise.resolve(undefined);
    });
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
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(true);
      return Promise.resolve(undefined);
    });
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
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(false);
      return Promise.resolve(undefined);
    });
    render(
      <StepHotkey onComplete={vi.fn()} binding={null} setBinding={vi.fn()} />
    );

    const continueBtn = screen.getByText("Continue");
    expect(continueBtn.hasAttribute("disabled")).toBe(true);
  });

  it("calls platform_register_hotkey on Continue click and completes", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(false);
      if (cmd === "platform_register_hotkey_cmd") return Promise.resolve(null);
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

    await waitFor(() => {
      expect(screen.getByText("Continue").hasAttribute("disabled")).toBe(false);
    });

    await act(async () => {
      fireEvent.click(screen.getByText("Continue"));
    });

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("platform_register_hotkey_cmd", {
        binding: "fn",
      });
    });

    expect(onComplete).toHaveBeenCalledOnce();
  });

  it("shows Input Monitoring error when fn registration fails", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(false);
      if (cmd === "platform_register_hotkey_cmd")
        return Promise.reject("Input Monitoring permission not granted");
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

    await waitFor(() => {
      expect(screen.getByText("Continue").hasAttribute("disabled")).toBe(false);
    });

    await act(async () => {
      fireEvent.click(screen.getByText("Continue"));
    });

    await waitFor(() => {
      expect(screen.getByText("Denied")).toBeTruthy();
      expect(screen.getByText("Open System Settings")).toBeTruthy();
    });

    expect(onComplete).not.toHaveBeenCalled();
  });

  it("does not show Input Monitoring error for ctrl+option+space binding", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_hotkey_conflicts_cmd") return Promise.resolve(false);
      return Promise.resolve(undefined);
    });
    const onComplete = vi.fn();

    render(
      <StepHotkey
        onComplete={onComplete}
        binding={{ mode: "ctrl+option+space" }}
        setBinding={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Continue").hasAttribute("disabled")).toBe(false);
    });

    expect(screen.queryByText("Input Monitoring")).toBeNull();
  });
});
