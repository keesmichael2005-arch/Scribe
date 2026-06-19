import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type AccessibilityStatus = "Granted" | "Denied" | "NotDetermined" | null;

interface StepAccessibilityProps {
  onComplete: () => void;
}

function StatusBadge({ status }: { status: AccessibilityStatus }) {
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

export default function StepAccessibility({ onComplete }: StepAccessibilityProps) {
  const [status, setStatus] = useState<AccessibilityStatus>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    const poll = () => {
      invoke<AccessibilityStatus>("platform_accessibility_status_cmd").then(setStatus);
    };
    poll();
    intervalRef.current = setInterval(poll, 1000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

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
        Accessibility access
      </h1>

      <div style={{ marginBottom: "16px" }}>
        <StatusBadge status={status} />
      </div>

      {status === "Denied" && (
        <button
          onClick={() => invoke("platform_open_accessibility_pane_cmd")}
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

      <p
        style={{
          fontSize: "12.5px",
          color: "var(--text-muted)",
          marginBottom: "16px",
        }}
      >
        Re-granting may be needed after every app update.
      </p>

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
