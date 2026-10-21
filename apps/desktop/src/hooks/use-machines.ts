import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { machineOf, machinesQuery } from "@/lib/machines";

export function useMachines() {
  const { data } = useQuery(machinesQuery);

  return useMemo(() => {
    const entries = data ?? [];

    return {
      all: entries.map(machineOf),
      connected: entries.filter(({ connected }) => connected).map(machineOf),
    };
  }, [data]);
}
