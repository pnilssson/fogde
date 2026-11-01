import { useSyncExternalStore } from "react";

const tick = 30_000;
const listeners = new Set<() => void>();
let now = Date.now();
let timer = 0;

function subscribe(listener: () => void) {
  listeners.add(listener);
  if (listeners.size === 1) {
    now = Date.now();
    timer = window.setInterval(() => {
      now = Date.now();
      for (const changed of listeners) {
        changed();
      }
    }, tick);
  }

  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) {
      window.clearInterval(timer);
    }
  };
}

export function useNow() {
  return useSyncExternalStore(subscribe, () => now);
}
