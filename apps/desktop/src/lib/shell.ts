import { queryOptions } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

export const shellQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "shell.get"],
    queryFn: () => machine.client.call("shell.get"),
  });

export const shellsQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "shell.list"],
    queryFn: () => machine.client.call("shell.list"),
  });
