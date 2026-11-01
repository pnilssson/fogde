import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

const notice = createStore<{ copied: string | undefined }>()(() => ({
  copied: undefined,
}));

let timer = 0;

export function useCopied(place: string) {
  return useStore(notice, ({ copied }) => copied === place);
}

export function flashCopied(place: string) {
  notice.setState({ copied: place });
  window.clearTimeout(timer);
  timer = window.setTimeout(() => {
    notice.setState({ copied: undefined });
  }, 1500);
}
