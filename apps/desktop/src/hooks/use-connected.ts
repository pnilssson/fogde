import { useEffect, useEffectEvent } from "react";

import type { Machine } from "@/lib/machines";

export function useConnected(
  machines: Machine[],
  listener: (machine: Machine) => void,
) {
  const handle = useEffectEvent(listener);

  useEffect(() => {
    const stops = machines.map((machine) => {
      let state = machine.client.getSnapshot().state;

      return machine.client.subscribe(() => {
        const next = machine.client.getSnapshot().state;
        if (next === "connected" && state !== "connected") {
          handle(machine);
        }

        state = next;
      });
    });

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines]);
}
