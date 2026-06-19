import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { egressDisclosureCopy, retentionDisclosure } from "../shared/index";

interface StepApiKeyProps {
  onComplete: () => void;
  apiKey: string;
  setKey: (k: string) => void;
}

export default function StepApiKey({ onComplete, apiKey, setKey }: StepApiKeyProps) {
  const [hasExisting, setHasExisting] = useState<boolean | null>(null);

  useEffect(() => {
    invoke<boolean>("has_api_key_cmd", { provider: "groq" }).then(setHasExisting);
  }, []);

  const handleContinue = async () => {
    await invoke("set_api_key_cmd", { provider: "groq", key: apiKey.trim() });
    onComplete();
  };

  return (
    <div>
      <h1
        style={{
          fontSize: "16px",
          fontWeight: 600,
          color: "var(--text-primary)",
          marginBottom: "12px",
        }}
      >
        API key
      </h1>

      {hasExisting ? (
        <div style={{ marginBottom: "16px" }}>
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "6px",
              fontSize: "12px",
              fontWeight: 500,
              padding: "4px 11px",
              borderRadius: "var(--r-badge)",
              color: "var(--text-primary)",
              background: "var(--success-soft)",
            }}
          >
            <span
              style={{
                width: "7px",
                height: "7px",
                borderRadius: "50%",
                background: "var(--success)",
                flexShrink: 0,
              }}
            />
            Key saved
          </span>
        </div>
      ) : (
        <input
          type="password"
          value={apiKey}
          onChange={(e) => setKey(e.target.value)}
          placeholder="Enter your Groq API key"
          style={{
            width: "100%",
            padding: "9px 12px",
            borderRadius: "var(--r-input)",
            border: "1px solid var(--border)",
            background: "var(--bg-primary)",
            color: "var(--text-primary)",
            fontFamily: "var(--font)",
            fontSize: "13.5px",
            outline: "none",
            marginBottom: "16px",
          }}
        />
      )}

      {hasExisting === false && (
        <>
          <div
            style={{
              background: "var(--accent-soft)",
              border: "1px solid color-mix(in srgb, var(--accent) 25%, transparent)",
              borderRadius: "9px",
              padding: "11px 13px",
              fontSize: "12.5px",
              color: "var(--text-secondary)",
              marginBottom: "8px",
            }}
          >
            <span style={{ fontWeight: 600 }}>Groq</span>{" "}
            {egressDisclosureCopy("groq")}
          </div>

          <p
            style={{
              fontSize: "12px",
              color: "var(--text-muted)",
              marginTop: "8px",
              marginBottom: "16px",
            }}
          >
            {retentionDisclosure("groq")}
          </p>
        </>
      )}

      <button
        onClick={handleContinue}
        disabled={hasExisting === null || (!hasExisting && apiKey.trim().length === 0)}
        style={{
          background:
            (hasExisting || apiKey.trim().length > 0)
              ? "var(--accent)"
              : "var(--bg-secondary)",
          color:
            (hasExisting || apiKey.trim().length > 0)
              ? "#FFFFFF"
              : "var(--text-disabled)",
          border:
            (hasExisting || apiKey.trim().length > 0)
              ? "none"
              : "1px solid var(--border)",
          padding: "9px 20px",
          borderRadius: "var(--r-btn)",
          fontFamily: "var(--font)",
          fontSize: "13.5px",
          fontWeight: 500,
          cursor:
            (hasExisting || apiKey.trim().length > 0)
              ? "pointer"
              : "not-allowed",
        }}
      >
        Continue
      </button>
    </div>
  );
}
