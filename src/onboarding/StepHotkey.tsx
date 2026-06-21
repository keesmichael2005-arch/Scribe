import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HotkeyBinding } from "./state";

interface StepHotkeyProps {
  onComplete: () => void;
  binding: HotkeyBinding | null;
  setBinding: (b: HotkeyBinding | null) => void;
}

export default function StepHotkey({ onComplete, binding }: StepHotkeyProps) {
  const [conflicts, setConflicts] = useState<boolean | null>(null);
  const [inputMonitoringError, setInputMonitoringError] = useState<
    string | null
  >(null);
  const [registering, setRegistering] = useState(false);

  useEffect(() => {
    if (binding) {
      invoke<boolean>("platform_hotkey_conflicts_cmd", {
        binding: binding.mode,
      }).then(setConflicts);
    }
  }, [binding]);

  const handleContinue = async () => {
    if (!binding) return;
    setRegistering(true);
    setInputMonitoringError(null);
    try {
      await invoke("platform_register_hotkey_cmd", {
        binding: binding.mode,
      });
      onComplete();
    } catch (err) {
      setInputMonitoringError(String(err));
    } finally {
      setRegistering(false);
    }
  };

  const isContinueDisabled =
    binding === null || conflicts !== false || registering;

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
        Hotkey
      </h1>

      {binding && (
        <p
          style={{
            fontSize: "13px",
            color: "var(--text-secondary)",
            marginBottom: "16px",
          }}
        >
          Hold the fn key (bottom-left of your keyboard) while speaking.
        </p>
      )}

      <div
        style={{
          background: "var(--bg-secondary)",
          borderRadius: "var(--r-card)",
          padding: "12px 16px",
          marginBottom: "16px",
        }}
      >
        <p
          style={{
            fontSize: "13px",
            color: "var(--text-secondary)",
            marginBottom: "8px",
          }}
        >
          Press Continue to select the fn key.
        </p>
        {binding && (
          <code
            style={{
              fontFamily: "var(--mono)",
              fontSize: "13px",
              color: "var(--text-primary)",
            }}
          >
            {binding.mode}
          </code>
        )}
      </div>

      {conflicts === true && (
        <p
          style={{
            fontSize: "12.5px",
            color: "var(--warning)",
            marginBottom: "16px",
          }}
        >
          This hotkey conflicts with an existing system shortcut.
        </p>
      )}

      {binding?.mode === "fn" && inputMonitoringError && (
        <div style={{ marginBottom: "16px" }}>
          <div style={{ marginBottom: "8px" }}>
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
                background: "var(--warning-soft)",
              }}
            >
              <span
                style={{
                  width: "7px",
                  height: "7px",
                  borderRadius: "50%",
                  background: "var(--warning)",
                  flexShrink: 0,
                }}
              />
              Denied
            </span>
          </div>
          <p
            style={{
              fontSize: "12.5px",
              color: "var(--text-muted)",
              marginBottom: "8px",
            }}
          >
            Input Monitoring permission is required for the fn key.
          </p>
          <button
            onClick={() =>
              invoke("platform_open_input_monitoring_pane_cmd")
            }
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
            }}
          >
            Open System Settings
          </button>
        </div>
      )}

      <button
        onClick={handleContinue}
        disabled={isContinueDisabled}
        style={{
          background: isContinueDisabled
            ? "var(--bg-secondary)"
            : "var(--accent)",
          color: isContinueDisabled ? "var(--text-disabled)" : "#FFFFFF",
          border: isContinueDisabled
            ? "1px solid var(--border)"
            : "none",
          padding: "9px 20px",
          borderRadius: "var(--r-btn)",
          fontFamily: "var(--font)",
          fontSize: "13.5px",
          fontWeight: 500,
          cursor: isContinueDisabled ? "not-allowed" : "pointer",
        }}
      >
        Continue
      </button>
    </div>
  );
}
