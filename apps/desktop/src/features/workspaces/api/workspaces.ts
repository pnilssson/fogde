import { type QueryClient, queryOptions } from "@tanstack/react-query";

import type { Terminal, TerminalState, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export const workspacesQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "workspaces.list"],
    queryFn: () => machine.client.call("workspaces.list"),
  });

function update(
  queries: QueryClient,
  machine: Machine,
  change: (workspace: Workspace) => Workspace,
) {
  queries.setQueryData(workspacesQuery(machine).queryKey, (current) =>
    current?.map(change),
  );
}

export function addTerminal(
  queries: QueryClient,
  machine: Machine,
  workspace: string,
  terminal: Terminal,
) {
  update(queries, machine, (entry) =>
    entry.id === workspace &&
    !entry.terminals.some(({ id }) => id === terminal.id)
      ? { ...entry, terminals: [...entry.terminals, terminal] }
      : entry,
  );
  void queries.invalidateQueries(workspacesQuery(machine));
}

export function applyState(
  queries: QueryClient,
  machine: Machine,
  state: TerminalState,
) {
  const { queryKey } = workspacesQuery(machine);
  const workspace = queries
    .getQueryData(queryKey)
    ?.find((entry) => entry.terminals.some(({ id }) => id === state.id));
  const before = workspace?.terminals.find(({ id }) => id === state.id);
  if (!workspace || !before) {
    void queries.invalidateQueries(workspacesQuery(machine));
    return undefined;
  }

  update(queries, machine, (entry) => ({
    ...entry,
    terminals: entry.terminals.map((terminal) =>
      terminal.id === state.id ? { ...terminal, ...state } : terminal,
    ),
  }));
  if (state.folder !== before.folder) {
    void queries.invalidateQueries(workspacesQuery(machine));
  }

  return { workspace, before };
}

export function removeTerminal(
  queries: QueryClient,
  machine: Machine,
  terminal: Terminal,
) {
  update(queries, machine, (workspace) => ({
    ...workspace,
    terminals: workspace.terminals.filter(({ id }) => id !== terminal.id),
  }));
  void queries.invalidateQueries(workspacesQuery(machine));
}
