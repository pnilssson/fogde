import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSyncExternalStore } from "react";

import type { Theme } from "@/generated/protocol";
import { call } from "@/lib/host";

const dark = window.matchMedia("(prefers-color-scheme: dark)");
const listeners = new Set<() => void>();
let chosen: Theme = "system";

function apply() {
  document.documentElement.classList.toggle(
    "dark",
    chosen === "dark" || (chosen === "system" && dark.matches),
  );
}

async function show(theme: Theme) {
  chosen = theme;
  apply();
  for (const listener of listeners) {
    listener();
  }

  await getCurrentWindow().setTheme(theme === "system" ? null : theme);
}

export async function followTheme() {
  apply();
  dark.addEventListener("change", apply);

  await show(await call("theme.get"));
}

export async function chooseTheme(theme: Theme) {
  // Save even when the window cannot follow, so the choice survives a restart.
  const shown = show(theme);
  await call("theme.set", theme);
  await shown;
}

function subscribe(listener: () => void) {
  listeners.add(listener);

  return () => {
    listeners.delete(listener);
  };
}

export function useTheme() {
  return useSyncExternalStore(subscribe, () => chosen);
}
