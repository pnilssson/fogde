import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useStore } from "zustand";
import { z } from "zod";
import { createStore } from "zustand/vanilla";

import type { Material, Theme } from "@/generated/protocol";
import { frame } from "@/lib/frame";
import { callOwn } from "@/lib/host";

const dark = window.matchMedia("(prefers-color-scheme: dark)");
const materials = z.enum(["translucent", "solid"]);
const key = "material";
const looks = createStore<{ theme: Theme; material: Material }>()(() => ({
  theme: "system",
  material: recall(),
}));

function recall(): Material {
  try {
    return materials.safeParse(localStorage.getItem(key)).data ?? "translucent";
  } catch {
    return "translucent";
  }
}

function remember(material: Material) {
  try {
    localStorage.setItem(key, material);
    return true;
  } catch {
    return false;
  }
}

function apply() {
  const { theme, material } = looks.getState();
  const root = document.documentElement;

  root.classList.toggle(
    "dark",
    theme === "dark" || (theme === "system" && dark.matches),
  );
  root.classList.toggle(
    "translucent",
    frame.translucent && material === "translucent",
  );
}

async function show(theme: Theme) {
  looks.setState({ theme });
  apply();

  await getCurrentWindow().setTheme(theme === "system" ? null : theme);
}

async function showMaterial(material: Material) {
  looks.setState({ material });
  remember(material);
  apply();

  await invoke("material", { material });
}

export async function followTheme() {
  apply();
  dark.addEventListener("change", apply);

  const [theme, material] = await Promise.all([
    callOwn("theme.get"),
    callOwn("theme.getMaterial"),
  ]);
  await Promise.all([show(theme), showMaterial(material)]);
}

export async function chooseTheme(theme: Theme) {
  const shown = show(theme);
  await callOwn("theme.set", theme);
  await shown;
}

export async function chooseMaterial(material: Material) {
  const shown = showMaterial(material);
  await callOwn("theme.setMaterial", material);
  await shown;
}

export function useTheme() {
  return useStore(looks, ({ theme }) => theme);
}

export function useMaterial() {
  return useStore(looks, ({ material }) => material);
}
