import React from "react";
import ReactDOM from "react-dom/client";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <div style={{
    position: "fixed",
    top: "40%",
    left: "50%",
    transform: "translate(-50%, -50%)",
    background: "rgba(30, 30, 30, 0.92)",
    color: "#fff",
    padding: "20px 32px",
    borderRadius: "12px",
    fontSize: "16px",
    fontFamily: "system-ui, sans-serif",
    pointerEvents: "none",
    userSelect: "none",
  }}>
    Scribe spike — panel visible
  </div>
);
