import { useQuery } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

const REFRESH = 15_000;

export function useSpaces(machine: Machine, paths: string[]) {
  return useQuery({
    queryKey: [machine.id, "monitor.spaces", paths],
    queryFn: () => machine.client.call("monitor.spaces", paths),
    enabled: paths.length > 0,
    refetchInterval: REFRESH,
  });
}
