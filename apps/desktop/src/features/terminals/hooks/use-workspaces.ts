import { useMutation, useQueryClient } from "@tanstack/react-query";

import { toast } from "@/components/ui/toast";
import { workspacesQuery } from "@/features/terminals/api/workspaces";
import { forget } from "@/features/terminals/lib/layouts";
import { selectWorkspace } from "@/features/terminals/lib/tiles";
import type {
  CreateWorkspaceParams,
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
  { terminal, workspace }: PlaceTerminalParams,
) {
  const moved = workspaces
    ?.flatMap(({ terminals }) => terminals)
    .find(({ id }) => id === terminal);

  return workspaces?.map((current) => {
    const terminals = current.terminals.filter(({ id }) => id !== terminal);

    return current.id === workspace && moved
      ? { ...current, terminals: [...terminals, moved] }
      : { ...current, terminals };
  });
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
      void queries.invalidateQueries(workspacesQuery(machine));
      toast.add({
        type: "error",
        title: "Could not move the terminal",
        description: error.message,
      });
    },
    onSettled: (_, __, { machine, terminal }) => {
      placing.delete(key(machine.id, terminal));
    },
  });
}

export function useDeleteWorkspace(machine: Machine, onDeleted: () => void) {
  const queries = useQueryClient();
  const { queryKey } = workspacesQuery(machine);

  return useMutation({
    mutationFn: (workspace: string) =>
      machine.client.call("workspaces.delete", workspace),
    onSuccess: (_, workspace) => {
      const home = queries.getQueryData(queryKey)?.[0];
      if (home) {
        selectWorkspace({ machine: machine.id, id: home.id });
      }

      forget({ machine: machine.id, id: workspace });
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
    mutationFn: (params: CreateWorkspaceParams) =>
      machine.client.call("workspaces.create", params),
    onSuccess: (workspace) => {
      queries.setQueryData(
        workspacesQuery(machine).queryKey,
        (current = []) => [...current, workspace],
      );
      selectWorkspace({ machine: machine.id, id: workspace.id });
      onCreated();
    },
  });
}
