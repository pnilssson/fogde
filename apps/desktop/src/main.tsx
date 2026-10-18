import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "@/app/app";
import { followWindowFocus } from "@/lib/focus";
import { showFrame } from "@/lib/frame";
import { followTheme } from "@/lib/theme";
import "./index.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

showFrame();
followWindowFocus();
// Wait for the saved theme, so the first paint does not flash the system one.
await followTheme().catch(() => undefined);

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
