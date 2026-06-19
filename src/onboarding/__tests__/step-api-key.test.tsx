import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import StepApiKey from "../StepApiKey";

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

describe("StepApiKey", () => {
  it("renders egress callout with Groq disclosure text", async () => {
    mockInvoke.mockResolvedValue(false);
    render(<StepApiKey onComplete={vi.fn()} apiKey="" setKey={vi.fn()} />);

    await waitFor(() => {
      expect(
        screen.getByText(/Your audio is sent to Groq for transcription/)
      ).toBeTruthy();
    });
  });

  it("renders retention disclosure text", async () => {
    mockInvoke.mockResolvedValue(false);
    render(<StepApiKey onComplete={vi.fn()} apiKey="" setKey={vi.fn()} />);

    await waitFor(() => {
      expect(
        screen.getByText(/Groq does not retain audio data after transcription/)
      ).toBeTruthy();
    });
  });

  it("input has type=password when no existing key", async () => {
    mockInvoke.mockResolvedValue(false);
    render(<StepApiKey onComplete={vi.fn()} apiKey="" setKey={vi.fn()} />);

    await waitFor(() => {
      const input = screen.getByPlaceholderText(
        "Enter your Groq API key"
      ) as HTMLInputElement;
      expect(input.type).toBe("password");
    });
  });

  it("disables Continue when key is empty and no existing key", async () => {
    mockInvoke.mockResolvedValue(false);
    render(<StepApiKey onComplete={vi.fn()} apiKey="" setKey={vi.fn()} />);

    await waitFor(() => {
      const continueBtn = screen.getByText("Continue");
      expect(continueBtn.hasAttribute("disabled")).toBe(true);
    });
  });

  it("enables Continue when key has a value", async () => {
    mockInvoke.mockResolvedValue(false);
    render(
      <StepApiKey onComplete={vi.fn()} apiKey="sk-test" setKey={vi.fn()} />
    );

    await waitFor(() => {
      const continueBtn = screen.getByText("Continue");
      expect(continueBtn.hasAttribute("disabled")).toBe(false);
    });
  });

  it("enables Continue when existing key is found", async () => {
    mockInvoke.mockResolvedValue(true);
    render(<StepApiKey onComplete={vi.fn()} apiKey="" setKey={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Key saved")).toBeTruthy();
      const continueBtn = screen.getByText("Continue");
      expect(continueBtn.hasAttribute("disabled")).toBe(false);
    });
  });

  it("calls set_api_key with trimmed value", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "has_api_key") return Promise.resolve(false);
      return Promise.resolve(undefined);
    });
    const onComplete = vi.fn();

    render(
      <StepApiKey
        onComplete={onComplete}
        apiKey="  sk-test  "
        setKey={vi.fn()}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Continue")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Continue"));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_api_key", {
        provider: "groq",
        key: "sk-test",
      });
    });

    expect(onComplete).toHaveBeenCalledOnce();
  });
});
