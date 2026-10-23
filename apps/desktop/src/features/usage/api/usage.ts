import { queryOptions } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

export const usageQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "usage.list"],
    queryFn: () => machine.client.call("usage.list"),
    staleTime: Infinity,
  });
