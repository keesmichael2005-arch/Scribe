import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function App() {
  const [loading, setLoading] = useState(true);
  const [onboardingComplete, setOnboardingComplete] = useState(true);
  const [Wizard, setWizard] = useState<React.ComponentType | null>(null);

  useEffect(() => {
    invoke<boolean>("is_onboarding_complete_cmd")
      .then((complete) => {
        setOnboardingComplete(complete);
        if (!complete) {
          import("./onboarding/index").then((mod) => {
            setWizard(() => mod.default);
          });
        }
        setLoading(false);
      })
      .catch(() => {
        setOnboardingComplete(false);
        import("./onboarding/index").then((mod) => {
          setWizard(() => mod.default);
        });
        setLoading(false);
      });
  }, []);

  if (loading) return null;

  if (!onboardingComplete && Wizard) {
    return <Wizard />;
  }

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        minHeight: "100vh",
        color: "var(--text-muted)",
        fontFamily: "var(--font)",
      }}
    >
      Scribe is running
    </div>
  );
}
