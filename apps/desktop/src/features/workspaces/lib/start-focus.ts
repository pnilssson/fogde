import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

const starting = createStore<{ focus: boolean }>()(() => ({
  focus: false,
}));

export function useStartFocus() {
  return useStore(starting, ({ focus }) => focus);
}

export function requestStartFocus(focus: boolean) {
  starting.setState({ focus });
}

export function startFocused() {
  starting.setState({ focus: false });
}
