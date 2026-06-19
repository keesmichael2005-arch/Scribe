interface StepIndicatorProps {
  currentStep: number;
  total?: number;
}

export default function StepIndicator({ currentStep, total = 6 }: StepIndicatorProps) {
  const bars = Array.from({ length: total }, (_, i) => i);

  return (
    <div style={{ display: "flex", gap: "6px" }}>
      {bars.map((i) => {
        const isCompleted = i < currentStep - 1;
        const isCurrent = i === currentStep - 1;
        const isAfterSlot4 = i >= 4;

        const bg = isAfterSlot4
          ? "var(--border)"
          : isCompleted || isCurrent
            ? "var(--accent)"
            : "var(--border)";

        return (
          <div
            key={i}
            style={{
              flex: 1,
              height: "4px",
              borderRadius: "2px",
              background: bg,
            }}
          />
        );
      })}
    </div>
  );
}
