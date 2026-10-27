import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

const list = createStore<{ shown: boolean }>()(() => ({ shown: true }));

export function useListShown() {
  return useStore(list, ({ shown }) => shown);
}

export function toggleList() {
  list.setState(({ shown }) => ({ shown: !shown }));
}
