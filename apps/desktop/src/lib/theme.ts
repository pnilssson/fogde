import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSyncExternalStore } from "react";

import type { Theme } from "@/generated/protocol";
import { callOwn } from "@/lib/host";

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

  await show(await callOwn("theme.get"));
}

export async function chooseTheme(theme: Theme) {
  const shown = show(theme);
  await callOwn("theme.set", theme);
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
