import { describe, it, expect } from "vitest";
import { egressDisclosureCopy } from "../index.js";

describe("egressDisclosureCopy", () => {
  it("returns the Groq egress disclosure", () => {
    expect(egressDisclosureCopy("groq")).toMatchInlineSnapshot(
      `"Your audio is sent to Groq for transcription. No processing happens on this device."`
    );
  });

  it("returns the OpenAI egress disclosure", () => {
    expect(egressDisclosureCopy("openai")).toMatchInlineSnapshot(
      `"Your audio is sent to OpenAI for transcription. No processing happens on this device."`
    );
  });
});
