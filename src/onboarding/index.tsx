import { invoke } from "@tauri-apps/api/core";
import { useOnboardingState } from "./state";
import StepIndicator from "./StepIndicator";
import StepMicrophone from "./StepMicrophone";
import StepAccessibility from "./StepAccessibility";
import StepApiKey from "./StepApiKey";
import StepHotkey from "./StepHotkey";

export default function OnboardingWizard() {
  const { state, dispatch } = useOnboardingState();

  const handleBack = () => {
    if (state.step > 1) {
      dispatch({ type: "SET_STEP", step: (state.step - 1) as 1 | 2 | 3 | 4 });
    }
  };

  const handleComplete = () => {
    invoke("mark_onboarding_complete").then(() => {
      invoke("close_onboarding");
    });
  };

  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        minHeight: "100vh",
        background: "var(--bg-primary)",
        padding: "24px",
      }}
    >
      <div
        style={{
          background: "var(--bg-elevation)",
          border: "1px solid var(--border)",
          borderRadius: "var(--r-card)",
          padding: "20px",
          boxShadow: "var(--card-shadow)",
          width: "100%",
          maxWidth: "420px",
        }}
      >
        <StepIndicator currentStep={state.step} total={6} />

        <div style={{ marginTop: "24px" }}>
          {state.step === 1 && (
            <StepMicrophone
              onComplete={() =>
                dispatch({ type: "SET_STEP", step: 2 })
              }
            />
          )}
          {state.step === 2 && (
            <StepAccessibility
              onComplete={() =>
                dispatch({ type: "SET_STEP", step: 3 })
              }
            />
          )}
          {state.step === 3 && (
            <StepApiKey
              onComplete={() => dispatch({ type: "SET_STEP", step: 4 })}
              apiKey={state.key}
              setKey={(k) => dispatch({ type: "SET_KEY", key: k })}
            />
          )}
          {state.step === 4 && (
            <StepHotkey
              onComplete={handleComplete}
              binding={state.binding}
              setBinding={(b) => dispatch({ type: "SET_BINDING", binding: b })}
            />
          )}
        </div>

        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            marginTop: "24px",
          }}
        >
          <button
            onClick={handleBack}
            disabled={state.step === 1}
            style={{
              background: "transparent",
              color:
                state.step === 1 ? "var(--text-disabled)" : "var(--accent)",
              border: "none",
              padding: "9px 18px",
              borderRadius: "var(--r-btn)",
              fontFamily: "var(--font)",
              fontSize: "13.5px",
              fontWeight: 500,
              cursor: state.step === 1 ? "not-allowed" : "pointer",
            }}
          >
            Back
          </button>
        </div>
      </div>
    </div>
  );
}
