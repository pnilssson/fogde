import { useQuery, useQueryClient } from "@tanstack/react-query";

import { isComputer, type Machine } from "@/lib/machines";

const KEEP = 150;

export interface Ping {
  time: number;
  millis: number;
}

export function useLatency(machine: Machine, enabled = true) {
  const queries = useQueryClient();
  const key = [machine.id, "latency"];

  return useQuery({
    queryKey: key,
    queryFn: async () => {
      const known = queries.getQueryData<Ping[]>(key) ?? [];
      const start = performance.now();
      await machine.client.call("health.ping");
      const millis = performance.now() - start;

      return [...known, { time: Date.now(), millis }].slice(-KEEP);
    },
    enabled: enabled && !isComputer(machine),
    refetchInterval: 2000,
    retry: false,
  });
}
