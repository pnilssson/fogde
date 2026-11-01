import {
  useIsMutating,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";

import { workspacesQuery } from "@/features/workspaces/api/workspaces";
import { mainCheckout, working } from "@/features/workspaces/lib/checkouts";
import { ask } from "@/features/workspaces/lib/confirm";
import { forget, store } from "@/features/workspaces/lib/layouts";
import { sameView } from "@/features/workspaces/lib/selection";
import { selectWorkspace } from "@/features/workspaces/lib/tiles";
import type {
  Checkout,
  CreateWorktreeParams,
  Workspace,
} from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { report } from "@/lib/report";
import { closePage } from "@/lib/page";

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
      closePage();
      selectWorkspace({ machine: machine.id, id: workspace, checkout: path });
    },
    onError: (error) => {
      report("Could not create the worktree", error);
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
      report("Could not remove the worktree", error);
    },
  });
}

export function useRemoveRequest(machine: Machine, workspace: Workspace) {
  const remove = useRemoveWorktree(machine, workspace);

  return (checkout: Checkout) => {
    if (working(workspace, checkout.path).length > 0) {
      ask({ kind: "remove", machine, workspace, checkout });
      return;
    }

    remove.mutate(checkout.path);
  };
}

export function useRemoval(
  machine: Machine,
  workspace: Workspace,
  checkout: Checkout,
) {
  const request = useRemoveRequest(machine, workspace);
  const pending =
    useIsMutating({
      mutationKey: removalKey(machine),
      predicate: ({ state }) => state.variables === checkout.path,
    }) > 0;

  return {
    pending,
    request: () => {
      request(checkout);
    },
  };
}
