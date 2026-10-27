import {
  useIsMutating,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { useState } from "react";

import { toast } from "@/components/ui/toast";
import { workspacesQuery } from "@/features/workspaces/api/workspaces";
import { mainCheckout, working } from "@/features/workspaces/lib/checkouts";
import { forget, store } from "@/features/workspaces/lib/layouts";
import { sameView } from "@/features/workspaces/lib/selection";
import { selectWorkspace } from "@/features/workspaces/lib/tiles";
import type { CreateWorktreeParams, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const removing = new Set<string>();

const removalKey = (machine: Machine) => [machine.id, "worktrees.remove"];

export function useCreateWorktree(
  machine: Machine,
  workspace: string,
  onCreated?: () => void,
) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: (params: CreateWorktreeParams) =>
      machine.client.call("worktrees.create", params),
    onSuccess: async (path) => {
      await queries.invalidateQueries(workspacesQuery(machine));
      onCreated?.();
      selectWorkspace({ machine: machine.id, id: workspace, checkout: path });
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not create the worktree",
        description: error.message,
      });
    },
  });
}

export function useRemoveWorktree(machine: Machine, workspace: Workspace) {
  const queries = useQueryClient();

  return useMutation({
    mutationKey: removalKey(machine),
    mutationFn: async (path: string) => {
      const main = mainCheckout(workspace);
      if (!main) {
        throw new Error("The repository's main checkout is missing.");
      }

      const key = `${machine.id}:${path}`;
      if (removing.has(key)) {
        return false;
      }

      removing.add(key);
      try {
        await machine.client.call("worktrees.remove", {
          repo: main.path,
          path,
        });
      } finally {
        removing.delete(key);
      }

      return true;
    },
    onSuccess: async (removed, path) => {
      if (!removed) {
        return;
      }

      const gone = { machine: machine.id, id: workspace.id, checkout: path };
      if (sameView(gone, store.getState().workspace)) {
        selectWorkspace({ machine: machine.id, id: workspace.id });
      }

      forget(gone);
      await queries.invalidateQueries(workspacesQuery(machine));
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not remove the worktree",
        description: error.message,
      });
    },
  });
}

export function useRemoval(
  machine: Machine,
  workspace: Workspace,
  path: string,
) {
  const remove = useRemoveWorktree(machine, workspace);
  const [confirming, setConfirming] = useState(false);
  const pending =
    useIsMutating({
      mutationKey: removalKey(machine),
      predicate: ({ state }) => state.variables === path,
    }) > 0;

  const request = () => {
    if (working(workspace, path).length > 0) {
      setConfirming(true);
      return;
    }

    remove.mutate(path);
  };

  return {
    pending,
    confirming,
    setConfirming,
    request,
  };
}
