import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

const settings = createStore<{ open: boolean }>()(() => ({ open: false }));

export function useSettingsOpen() {
  return useStore(settings, ({ open }) => open);
}

export function setSettingsOpen(open: boolean) {
  settings.setState({ open });
}

export function toggleSettings() {
  settings.setState(({ open }) => ({ open: !open }));
}
