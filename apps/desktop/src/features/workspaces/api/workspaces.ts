import { type QueryClient, queryOptions } from "@tanstack/react-query";

import { terminalsOf } from "@/features/workspaces/lib/tile-kinds";
import type {
  Terminal,
  TerminalState,
  Tile,
  Workspace,
} from "@/generated/protocol";
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

export function addTile(
  queries: QueryClient,
  machine: Machine,
  workspace: string,
  tile: Tile,
) {
  update(queries, machine, (entry) =>
    entry.id === workspace && !entry.tiles.some(({ id }) => id === tile.id)
      ? { ...entry, tiles: [...entry.tiles, tile] }
      : entry,
  );
}

export function changeTile(
  queries: QueryClient,
  machine: Machine,
  id: string,
  change: (tile: Tile) => Tile,
) {
  update(queries, machine, (workspace) => ({
    ...workspace,
    tiles: workspace.tiles.map((tile) =>
      tile.id === id ? change(tile) : tile,
    ),
  }));
}

export function removeTile(queries: QueryClient, machine: Machine, id: string) {
  update(queries, machine, (workspace) => ({
    ...workspace,
    tiles: workspace.tiles.filter((tile) => tile.id !== id),
  }));
}

export function addTerminal(
  queries: QueryClient,
  machine: Machine,
  workspace: string,
  terminal: Terminal,
) {
  addTile(queries, machine, workspace, { kind: "terminal", ...terminal });
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
    ?.find((entry) => terminalsOf(entry).some(({ id }) => id === state.id));
  const before = workspace
    ? terminalsOf(workspace).find(({ id }) => id === state.id)
    : undefined;
  if (!workspace || !before) {
    void queries.invalidateQueries(workspacesQuery(machine));
    return undefined;
  }

  changeTile(queries, machine, state.id, (tile) =>
    tile.kind === "terminal" ? { ...tile, ...state } : tile,
  );
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
  removeTile(queries, machine, terminal.id);
  void queries.invalidateQueries(workspacesQuery(machine));
}
