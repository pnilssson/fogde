import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "@/app/app";
import { followWindowFocus } from "@/lib/focus";
import { followTheme } from "@/lib/theme";
import "./index.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

followWindowFocus();
await followTheme().catch(() => undefined);

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
