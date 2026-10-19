import { queryOptions } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

export const remoteReposQuery = (machine: Machine, host: string) =>
  queryOptions({
    queryKey: [machine.id, "repos.list", host],
    queryFn: () => machine.client.call("repos.list", host),
  });
