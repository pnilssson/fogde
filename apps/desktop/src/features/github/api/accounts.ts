import { queryOptions } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

export const accountsQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "github.accounts"],
    queryFn: () => machine.client.call("github.accounts"),
  });
