import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HotkeyBinding } from "./state";

interface StepHotkeyProps {
  onComplete: () => void;
  binding: HotkeyBinding | null;
  setBinding: (b: HotkeyBinding | null) => void;
}

export default function StepHotkey({ onComplete, binding, setBinding }: StepHotkeyProps) {
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

  const options: { mode: "fn" | "ctrl+option+space"; label: string; hint: string }[] = [
    { mode: "fn", label: "fn", hint: "Globe key, bottom-left of keyboard" },
    { mode: "ctrl+option+space", label: "⌃⌥Space", hint: "Ctrl + Option + Space" },
  ];

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

      <p
        style={{
          fontSize: "13px",
          color: "var(--text-secondary)",
          marginBottom: "14px",
        }}
      >
        Hold this key while speaking to record.
      </p>

      <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginBottom: "16px" }}>
        {options.map((opt) => {
          const selected = binding?.mode === opt.mode;
          return (
            <button
              key={opt.mode}
              onClick={() => { setBinding({ mode: opt.mode }); setConflicts(null); setInputMonitoringError(null); }}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "12px",
                background: selected ? "var(--accent-soft)" : "var(--bg-secondary)",
                border: selected
                  ? "1.5px solid color-mix(in srgb, var(--accent) 40%, transparent)"
                  : "1.5px solid var(--border)",
                borderRadius: "var(--r-card)",
                padding: "10px 14px",
                cursor: "pointer",
                textAlign: "left",
                width: "100%",
              }}
            >
              <span
                style={{
                  width: "14px",
                  height: "14px",
                  borderRadius: "50%",
                  border: selected ? "4px solid var(--accent)" : "2px solid var(--border)",
                  background: selected ? "var(--accent)" : "transparent",
                  flexShrink: 0,
                  boxSizing: "border-box",
                }}
              />
              <span>
                <code style={{ fontFamily: "var(--mono)", fontSize: "13px", color: "var(--text-primary)", fontWeight: 600 }}>
                  {opt.label}
                </code>
                <span style={{ fontSize: "12px", color: "var(--text-muted)", marginLeft: "8px" }}>
                  {opt.hint}
                </span>
              </span>
            </button>
          );
        })}
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
