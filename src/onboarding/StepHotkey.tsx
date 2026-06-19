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

  useEffect(() => {
    if (binding) {
      invoke<boolean>("platform_hotkey_conflicts", { binding: binding.mode }).then(
        setConflicts
      );
    }
  }, [binding]);

  const handleContinue = async () => {
    if (!binding) return;
    await invoke("platform_register_hotkey", { binding: binding.mode });
    onComplete();
  };

  const isContinueDisabled = binding === null || conflicts === true;

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

      <button
        onClick={handleContinue}
        disabled={isContinueDisabled}
        style={{
          background: isContinueDisabled ? "var(--bg-secondary)" : "var(--accent)",
          color: isContinueDisabled ? "var(--text-disabled)" : "#FFFFFF",
          border: isContinueDisabled ? "1px solid var(--border)" : "none",
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
