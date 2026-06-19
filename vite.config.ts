import path from "path";
import { defineConfig } from "vitest/config";

let reactPlugin: any = undefined;
try {
  reactPlugin = (await import("@vitejs/plugin-react")).default;
} catch {}

export default defineConfig({
  plugins: reactPlugin ? [reactPlugin()] : [],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: "dist",
  },
  resolve: {
    alias: {
      "@": path.resolve("./src"),
    },
    conditions: ["development", "browser"],
  },
  define: {
    "process.env.NODE_ENV": JSON.stringify("development"),
  },
  test: {
    environmentMatchGlobs: [
      ["src/**/__tests__/**/*.test.tsx", "jsdom"],
    ],
    include: ["src/**/__tests__/**/*.test.ts", "src/**/__tests__/**/*.test.tsx"],
    environment: "node",
  },
});
