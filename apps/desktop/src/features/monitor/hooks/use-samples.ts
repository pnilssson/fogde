import { useQuery, useQueryClient } from "@tanstack/react-query";

import type { Sample } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const SPAN = 300_000;

export interface Reading extends Sample {
  time: number;
}

export function useSamples(machine: Machine, enabled = true) {
  const queries = useQueryClient();
  const key = [machine.id, "monitor.samples"];

  return useQuery({
    queryKey: key,
    queryFn: async () => {
      const known = queries.getQueryData<Reading[]>(key) ?? [];
      const fresh = await machine.client.call("monitor.samples", {
        since: known.at(-1)?.at ?? 0,
      });
      const now = Date.now();
      const skew = now - (fresh.at(-1)?.at ?? now);

      return [
        ...known,
        ...fresh.map((sample) => ({ ...sample, time: sample.at + skew })),
      ].filter(({ time }) => time > now - SPAN);
    },
    enabled,
    refetchInterval: 1000,
    retry: false,
  });
}
