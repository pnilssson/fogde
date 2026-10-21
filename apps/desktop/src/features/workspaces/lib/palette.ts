import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

const palette = createStore<{ open: boolean }>()(() => ({ open: false }));

export function usePaletteOpen() {
  return useStore(palette, ({ open }) => open);
}

export function setPaletteOpen(open: boolean) {
  palette.setState({ open });
}

export function togglePalette() {
  palette.setState(({ open }) => ({ open: !open }));
}
