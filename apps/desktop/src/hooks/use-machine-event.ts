import { useEffect, useEffectEvent } from "react";

import type { Events } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export function useMachineEvent<E extends keyof Events>(
  machines: Machine[],
  event: E,
  listener: (machine: Machine, params: Events[E]) => void,
) {
  const handle = useEffectEvent(listener);

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.listen(event, (params) => {
        handle(machine, params);
      }),
    );

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, event]);
}
