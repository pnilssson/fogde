import { queryOptions } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

export const notesQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "notes.list"],
    queryFn: () => machine.client.call("notes.list"),
  });

export const noteQuery = (
  machine: Pick<Machine, "id" | "client">,
  path: string,
) =>
  queryOptions({
    queryKey: [machine.id, "notes.read", path],
    queryFn: () => machine.client.call("notes.read", path),
  });
