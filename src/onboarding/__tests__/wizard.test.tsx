import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import OnboardingWizard from "../index";
import StepMicrophone from "../StepMicrophone";
import StepAccessibility from "../StepAccessibility";
import StepIndicator from "../StepIndicator";

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
  vi.useRealTimers();
});

// --- StepMicrophone tests (rendered directly) ---
describe("StepMicrophone", () => {
  it("disables Continue when micStatus is NotDetermined", async () => {
    mockInvoke.mockResolvedValue("NotDetermined");
    render(<StepMicrophone onComplete={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Microphone access")).toBeTruthy();
    });

    const continueBtn = screen.getByText("Continue");
    expect(continueBtn.hasAttribute("disabled")).toBe(true);
  });

  it("disables Continue and shows Open System Settings when micStatus is Denied", async () => {
    mockInvoke.mockResolvedValue("Denied");
    render(<StepMicrophone onComplete={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Open System Settings")).toBeTruthy();
    });

    const continueBtn = screen.getByText("Continue");
    expect(continueBtn.hasAttribute("disabled")).toBe(true);
  });

  it("enables Continue when micStatus is Granted", async () => {
    mockInvoke.mockResolvedValue("Granted");
    const onComplete = vi.fn();
    render(<StepMicrophone onComplete={onComplete} />);

    await waitFor(() => {
      expect(screen.getByText("Allowed")).toBeTruthy();
    });

    const continueBtn = screen.getByText("Continue");
    expect(continueBtn.hasAttribute("disabled")).toBe(false);

    fireEvent.click(continueBtn);
    expect(onComplete).toHaveBeenCalledOnce();
  });

  it("calls platform_open_microphone_pane when Open System Settings is clicked", async () => {
    mockInvoke.mockResolvedValue("Denied");
    render(<StepMicrophone onComplete={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Open System Settings")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Open System Settings"));
    expect(mockInvoke).toHaveBeenCalledWith("platform_open_microphone_pane");
  });
});

// --- StepAccessibility tests (rendered directly) ---
describe("StepAccessibility", () => {
  it("contains the re-granting notice text", async () => {
    mockInvoke.mockResolvedValue("NotDetermined");
    render(<StepAccessibility onComplete={vi.fn()} />);

    await waitFor(() => {
      expect(
        screen.getByText(
          "Re-granting may be needed after every app update."
        )
      ).toBeTruthy();
    });
  });

  it("shows Open System Settings when accessibilityStatus is Denied", async () => {
    mockInvoke.mockResolvedValue("Denied");
    render(<StepAccessibility onComplete={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Open System Settings")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Open System Settings"));
    expect(mockInvoke).toHaveBeenCalledWith("platform_open_accessibility_pane");
  });

  it("enables Continue when accessibilityStatus is Granted", async () => {
    mockInvoke.mockResolvedValue("Granted");
    const onComplete = vi.fn();
    render(<StepAccessibility onComplete={onComplete} />);

    await waitFor(() => {
      expect(screen.getByText("Allowed")).toBeTruthy();
    });

    const continueBtn = screen.getByText("Continue");
    expect(continueBtn.hasAttribute("disabled")).toBe(false);

    fireEvent.click(continueBtn);
    expect(onComplete).toHaveBeenCalledOnce();
  });

  it("disables Continue when accessibilityStatus is Denied", async () => {
    mockInvoke.mockResolvedValue("Denied");
    render(<StepAccessibility onComplete={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Open System Settings")).toBeTruthy();
    });

    const continueBtn = screen.getByText("Continue");
    expect(continueBtn.hasAttribute("disabled")).toBe(true);
  });
});

// --- StepIndicator tests ---
describe("StepIndicator", () => {
  it("renders 6 bars", () => {
    const { container } = render(
      <StepIndicator currentStep={1} total={6} />
    );
    const bars = container.querySelectorAll('[style*="flex: 1"]');
    expect(bars).toHaveLength(6);
  });

  it("styles bars 5 and 6 with --border through Sprint 1 when currentStep=1", () => {
    const { container } = render(
      <StepIndicator currentStep={1} total={6} />
    );
    const bars = container.querySelectorAll('[style*="flex: 1"]');
    expect(bars[4].getAttribute("style")).toContain("var(--border)");
    expect(bars[5].getAttribute("style")).toContain("var(--border)");
  });

  it("styles bars 5 and 6 with --border even when currentStep=6", () => {
    const { container } = render(
      <StepIndicator currentStep={6} total={6} />
    );
    const bars = container.querySelectorAll('[style*="flex: 1"]');
    expect(bars[4].getAttribute("style")).toContain("var(--border)");
    expect(bars[5].getAttribute("style")).toContain("var(--border)");
  });
});

// --- Wizard integration tests ---
describe("OnboardingWizard", () => {
  it("renders Step 1 initially", async () => {
    mockInvoke.mockResolvedValue("NotDetermined");
    const { container } = render(<OnboardingWizard />);

    await waitFor(() => {
      const headings = container.querySelectorAll("h1");
      expect(headings).toHaveLength(1);
      expect(headings[0].textContent).toBe("Microphone access");
    });
  });

  it("back button is disabled on step 1", async () => {
    mockInvoke.mockResolvedValue("NotDetermined");
    render(<OnboardingWizard />);

    await waitFor(() => {
      const backBtns = screen.getAllByText("Back");
      expect(backBtns).toHaveLength(1);
      expect(backBtns[0].hasAttribute("disabled")).toBe(true);
    });
  });

  it("back button navigates from step 2 to step 1", async () => {
    mockInvoke.mockResolvedValue("Granted");
    render(<OnboardingWizard />);

    await waitFor(() => {
      expect(screen.getByText("Allowed")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Continue"));

    mockInvoke.mockResolvedValue("Denied");

    await waitFor(() => {
      expect(screen.getByText("Accessibility access")).toBeTruthy();
    });

    const backBtns = screen.getAllByText("Back");
    expect(backBtns).toHaveLength(1);
    expect(backBtns[0].hasAttribute("disabled")).toBe(false);
    fireEvent.click(backBtns[0]);

    await waitFor(() => {
      expect(screen.getByText("Microphone access")).toBeTruthy();
    });
  });

  it("back button preserves entered state across steps", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "platform_microphone_status") return Promise.resolve("Granted");
      if (cmd === "platform_accessibility_status") return Promise.resolve("Granted");
      if (cmd === "has_api_key") return Promise.resolve(false);
      if (cmd === "platform_hotkey_conflicts") return Promise.resolve(false);
      return Promise.resolve(undefined);
    });
    render(<OnboardingWizard />);

    await waitFor(() => {
      expect(screen.getByText("Microphone access")).toBeTruthy();
      expect(screen.getByText("Allowed")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Continue"));

    await waitFor(() => {
      expect(screen.getByText("Accessibility access")).toBeTruthy();
      expect(screen.getByText("Allowed")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Continue"));

    await waitFor(() => {
      expect(screen.getByText("API key")).toBeTruthy();
    });

    const input = screen.getByPlaceholderText("Enter your Groq API key");
    fireEvent.change(input, { target: { value: "sk-test-key" } });

    fireEvent.click(screen.getByText("Back"));

    await waitFor(() => {
      expect(screen.getByText("Accessibility access")).toBeTruthy();
      expect(screen.getByText("Allowed")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Continue"));

    await waitFor(() => {
      expect(screen.getByText("API key")).toBeTruthy();
    });

    const inputAgain = screen.getByPlaceholderText(
      "Enter your Groq API key"
    ) as HTMLInputElement;
    expect(inputAgain.value).toBe("sk-test-key");
  });
});
