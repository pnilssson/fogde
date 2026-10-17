import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "@/app/app";
import { showFrame } from "@/lib/frame";
import { followSystemTheme } from "@/lib/theme";
import "./index.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("missing #root element");
}

showFrame();
followSystemTheme();

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
