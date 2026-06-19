import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type MicStatus = "Granted" | "Denied" | "NotDetermined" | null;

interface StepMicrophoneProps {
  onComplete: () => void;
}

function StatusBadge({ status }: { status: MicStatus }) {
  if (!status) return null;

  const config: Record<string, { dot: string; bg: string; label: string }> = {
    Granted: { dot: "var(--success)", bg: "var(--success-soft)", label: "Allowed" },
    Denied: { dot: "var(--warning)", bg: "var(--warning-soft)", label: "Denied" },
  };

  if (status === "NotDetermined") {
    return (
      <span
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: "6px",
          fontSize: "12px",
          fontWeight: 500,
          padding: "4px 11px",
          borderRadius: "var(--r-badge)",
          color: "var(--text-muted)",
          background: "var(--bg-secondary)",
        }}
      >
        <span
          style={{
            width: "7px",
            height: "7px",
            borderRadius: "50%",
            background: "var(--text-muted)",
            flexShrink: 0,
          }}
        />
        Not determined
      </span>
    );
  }

  const c = config[status];

  return (
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
        background: c.bg,
      }}
    >
      <span
        style={{
          width: "7px",
          height: "7px",
          borderRadius: "50%",
          background: c.dot,
          flexShrink: 0,
        }}
      />
      {c.label}
    </span>
  );
}

export default function StepMicrophone({ onComplete }: StepMicrophoneProps) {
  const [status, setStatus] = useState<MicStatus>(null);

  useEffect(() => {
    invoke<MicStatus>("platform_microphone_status").then(setStatus);
  }, []);

  const handleGrant = async () => {
    const result = await invoke<MicStatus>("request_microphone_permission");
    setStatus(result);
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
        Microphone access
      </h1>

      <div style={{ marginBottom: "16px" }}>
        <StatusBadge status={status} />
      </div>

      {status === "Denied" && (
        <button
          onClick={() => invoke("platform_open_microphone_pane")}
          style={{
            background: "var(--bg-secondary)",
            color: "var(--text-primary)",
            border: "1px solid var(--border)",
            padding: "7px 16px",
            borderRadius: "var(--r-btn)",
            fontFamily: "var(--font)",
            fontSize: "13px",
            fontWeight: 500,
            cursor: "pointer",
            marginBottom: "16px",
          }}
        >
          Open System Settings
        </button>
      )}

      {status === "NotDetermined" && (
        <div>
          <p
            style={{
              fontSize: "13px",
              color: "var(--text-secondary)",
              marginBottom: "12px",
            }}
          >
            Microphone access is required for dictation
          </p>
          <button
            onClick={handleGrant}
            style={{
              background: "var(--accent)",
              color: "#FFFFFF",
              border: "none",
              padding: "9px 20px",
              borderRadius: "var(--r-btn)",
              fontFamily: "var(--font)",
              fontSize: "13.5px",
              fontWeight: 500,
              cursor: "pointer",
              marginBottom: "16px",
            }}
          >
            Grant Permission
          </button>
        </div>
      )}

      <button
        onClick={onComplete}
        disabled={status !== "Granted"}
        style={{
          background: status === "Granted" ? "var(--accent)" : "var(--bg-secondary)",
          color: status === "Granted" ? "#FFFFFF" : "var(--text-disabled)",
          border: status === "Granted" ? "none" : "1px solid var(--border)",
          padding: "9px 20px",
          borderRadius: "var(--r-btn)",
          fontFamily: "var(--font)",
          fontSize: "13.5px",
          fontWeight: 500,
          cursor: status === "Granted" ? "pointer" : "not-allowed",
        }}
      >
        Continue
      </button>
    </div>
  );
}
