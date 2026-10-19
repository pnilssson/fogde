import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { machineOf, machinesQuery } from "@/lib/machines";

export function useMachines() {
  const { data } = useQuery(machinesQuery);

  return useMemo(() => (data ?? []).map(machineOf), [data]);
}
