import { type QueryClient, queryOptions } from "@tanstack/react-query";

import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export const workspacesQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "workspaces.list"],
    queryFn: () => machine.client.call("workspaces.list"),
  });

export function addTerminal(
  queries: QueryClient,
  machine: Machine,
  workspace: string,
  terminal: Terminal,
) {
  queries.setQueryData(workspacesQuery(machine).queryKey, (current) =>
    current?.map((entry) =>
      entry.id === workspace
        ? { ...entry, terminals: [...entry.terminals, terminal] }
        : entry,
    ),
  );
  void queries.invalidateQueries(workspacesQuery(machine));
}

export function removeTerminal(
  queries: QueryClient,
  machine: Machine,
  terminal: Terminal,
) {
  queries.setQueryData(workspacesQuery(machine).queryKey, (current) =>
    current?.map((workspace) => ({
      ...workspace,
      terminals: workspace.terminals.filter(({ id }) => id !== terminal.id),
    })),
  );
  void queries.invalidateQueries(workspacesQuery(machine));
}
