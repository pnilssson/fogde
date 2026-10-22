import path from "node:path";

import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

import tauri from "./src-tauri/tauri.conf.json" with { type: "json" };

const csp = `${tauri.app.security.csp.replace("connect-src", "connect-src 'self'")}; script-src 'self' 'unsafe-inline'`;

export default defineConfig({
  plugins: [
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
      generatedRouteTree: "./src/generated/route-tree.gen.ts",
    }),
    react(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    headers: {
      "Content-Security-Policy": csp,
    },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
