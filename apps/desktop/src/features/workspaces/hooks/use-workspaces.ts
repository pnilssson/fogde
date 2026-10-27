import {
  type QueryClient,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";

import { toast } from "@/components/ui/toast";
import {
  addTile,
  changeTile,
  removeTile,
  workspacesQuery,
} from "@/features/workspaces/api/workspaces";
import { forget } from "@/features/workspaces/lib/layouts";
import { placeTile, type Spot } from "@/features/workspaces/lib/opening";
import { arranged } from "@/features/workspaces/lib/order";
import type { View } from "@/features/workspaces/lib/selection";
import { terminalsOf } from "@/features/workspaces/lib/tile-kinds";
import {
  leaveWorkspace,
  selectWorkspace,
} from "@/features/workspaces/lib/tiles";
import type {
  MoveTileParams,
  PlaceTerminalParams,
  Workspace,
} from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const placing = new Set<string>();

function key(machine: string, terminal: string) {
  return `${machine}:${terminal}`;
}

export function isPlacing(machine: string, terminal: string) {
  return placing.has(key(machine, terminal));
}

function move(
  workspaces: Workspace[] | undefined,
  { terminal, workspace, checkout }: PlaceTerminalParams,
) {
  const moved = workspaces
    ?.flatMap(terminalsOf)
    .find(({ id }) => id === terminal);
  const stays = workspaces?.some(
    (current) =>
      current.id === workspace &&
      terminalsOf(current).some(
        ({ id, checkout: placed }) => id === terminal && placed === checkout,
      ),
  );
  if (stays) {
    return workspaces;
  }

  return workspaces?.map((current) => {
    const tiles = current.tiles.filter(({ id }) => id !== terminal);

    return current.id === workspace && moved
      ? { ...current, tiles: [...tiles, { ...moved, checkout }] }
      : { ...current, tiles };
  });
}

function failed(
  queries: QueryClient,
  machine: Machine,
  title: string,
  error: Error,
) {
  void queries.invalidateQueries(workspacesQuery(machine));
  toast.add({ type: "error", title, description: error.message });
}

interface Place extends PlaceTerminalParams {
  machine: Machine;
}

export function usePlaceTerminal() {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: ({ machine, ...params }: Place) =>
      machine.client.call("workspaces.place", params),
    onMutate: ({ machine, ...params }) => {
      placing.add(key(machine.id, params.terminal));
      queries.setQueryData(workspacesQuery(machine).queryKey, (current) =>
        move(current, params),
      );
    },
    onError: (error, { machine }) => {
      failed(queries, machine, "Could not move the terminal", error);
    },
    onSettled: (_, __, { machine, terminal }) => {
      placing.delete(key(machine.id, terminal));
    },
  });
}

function arrange(
  workspaces: Workspace[] | undefined,
  { tile, before }: MoveTileParams,
) {
  return workspaces?.map((workspace) => {
    const moving = workspace.tiles.find(({ id }) => id === tile);
    if (!moving) {
      return workspace;
    }

    const target = workspace.tiles.find(({ id }) => id === before) ?? null;

    return { ...workspace, tiles: arranged(workspace.tiles, moving, target) };
  });
}

interface Move extends MoveTileParams {
  machine: Machine;
}

export function useMoveTile() {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: ({ machine, ...params }: Move) =>
      machine.client.call("workspaces.move", params),
    onMutate: async ({ machine, ...params }) => {
      await queries.cancelQueries(workspacesQuery(machine));
      queries.setQueryData(workspacesQuery(machine).queryKey, (current) =>
        arrange(current, params),
      );
    },
    onError: (error, { machine }) => {
      failed(queries, machine, "Could not move the tile", error);
    },
    onSettled: (_, __, { machine }) =>
      queries.invalidateQueries(workspacesQuery(machine)),
  });
}

interface OpenNote {
  machine: Machine;
  view: View;
  note?: string | undefined;
  spot?: Spot | undefined;
}

export function useOpenNote() {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: ({ machine, view, note }: OpenNote) =>
      machine.client.call("workspaces.openNote", {
        workspace: view.id,
        checkout: view.checkout ?? null,
        note: note ?? null,
      }),
    onMutate: ({ machine }) => queries.cancelQueries(workspacesQuery(machine)),
    onSuccess: (id, { machine, view, note, spot }) => {
      addTile(queries, machine, view.id, {
        kind: "note",
        id,
        checkout: view.checkout ?? null,
        note: note ?? null,
      });
      placeTile(view, "note", id, spot);
    },
    onError: (error, { machine }) => {
      failed(queries, machine, "Could not open the notes", error);
    },
    onSettled: (_, __, { machine }) =>
      queries.invalidateQueries(workspacesQuery(machine)),
  });
}

interface ShowNote {
  machine: Machine;
  tile: string;
  note: string | undefined;
}

export function useShowNote() {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: ({ machine, tile, note }: ShowNote) =>
      machine.client.call("workspaces.showNote", { tile, note: note ?? null }),
    onMutate: async ({ machine, tile, note }) => {
      await queries.cancelQueries(workspacesQuery(machine));
      changeTile(queries, machine, tile, (current) =>
        current.kind === "note" ? { ...current, note: note ?? null } : current,
      );
    },
    onError: (error, { machine }) => {
      failed(queries, machine, "Could not open the note", error);
    },
    onSettled: (_, __, { machine }) =>
      queries.invalidateQueries(workspacesQuery(machine)),
  });
}

interface CloseTile {
  machine: Machine;
  tile: string;
}

export function useCloseTile() {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: ({ machine, tile }: CloseTile) =>
      machine.client.call("workspaces.closeTile", tile),
    onMutate: async ({ machine, tile }) => {
      await queries.cancelQueries(workspacesQuery(machine));
      removeTile(queries, machine, tile);
    },
    onError: (error, { machine }) => {
      failed(queries, machine, "Could not close the tile", error);
    },
    onSettled: (_, __, { machine }) =>
      queries.invalidateQueries(workspacesQuery(machine)),
  });
}

export function useDeleteWorkspace(machine: Machine, onDeleted: () => void) {
  const queries = useQueryClient();
  const { queryKey } = workspacesQuery(machine);

  return useMutation({
    mutationFn: (workspace: string) =>
      machine.client.call("workspaces.delete", workspace),
    onSuccess: (_, workspace) => {
      const gone = { machine: machine.id, id: workspace };
      leaveWorkspace(gone);
      forget(gone);
      queries.setQueryData(queryKey, (current) =>
        current?.filter(({ id }) => id !== workspace),
      );
      onDeleted();
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not delete the workspace",
        description: error.message,
      });
    },
    onSettled: () => queries.invalidateQueries(workspacesQuery(machine)),
  });
}

export function useCreateWorkspace(machine: Machine, onCreated: () => void) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: (folder: string) =>
      machine.client.call("workspaces.create", folder),
    onSuccess: (workspace) => {
      queries.setQueryData(workspacesQuery(machine).queryKey, (current = []) =>
        current.some(({ id }) => id === workspace.id)
          ? current
          : [...current, workspace],
      );
      selectWorkspace({ machine: machine.id, id: workspace.id });
      onCreated();
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not create the workspace",
        description: error.message,
      });
    },
  });
}
