import { useSyncExternalStore } from "react";

import type { Machine } from "@/lib/machines";

export function useConnection(machine: Machine) {
  return useSyncExternalStore(
    machine.client.subscribe,
    machine.client.getSnapshot,
  );
}
