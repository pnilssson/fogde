import { focusManager } from "@tanstack/react-query";
import { useEffect, useSyncExternalStore } from "react";

import { useTiles } from "@/features/workspaces/lib/layouts";
import type { Machine } from "@/lib/machines";

export function useViewing(machines: Machine[]) {
  const active = useTiles((tiles) => tiles.active);
  const focused = useSyncExternalStore(
    (listener) => focusManager.subscribe(listener),
    () => focusManager.isFocused(),
  );

  useEffect(() => {
    const viewing = focused ? active : undefined;
    const stops = machines.map((machine) => {
      const send = () => {
        if (machine.client.getSnapshot().state !== "connected") {
          return;
        }

        const id = viewing?.machine === machine.id ? viewing.id : null;
        machine.client.call("terminals.view", id).catch(() => undefined);
      };

      send();
      return machine.client.subscribe(send);
    });

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, active, focused]);
}
