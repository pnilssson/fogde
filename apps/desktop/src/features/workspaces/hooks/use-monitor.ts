import { useMutation, useQueryClient } from "@tanstack/react-query";

import { toast } from "@/components/ui/toast";
import { addTile, workspacesQuery } from "@/features/workspaces/api/workspaces";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { placeTile } from "@/features/workspaces/lib/opening";
import type { View } from "@/features/workspaces/lib/selection";
import { selectWorkspace } from "@/features/workspaces/lib/tiles";
import type { Machine } from "@/lib/machines";

type Groups = ReturnType<typeof useGroups>;

interface OpenMonitor {
  machine: Machine;
  view: View;
}

export function useOpenMonitor() {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: ({ machine, view }: OpenMonitor) =>
      machine.client.call("workspaces.openMonitor", {
        workspace: view.id,
        checkout: view.checkout ?? null,
      }),
    onMutate: ({ machine }) => queries.cancelQueries(workspacesQuery(machine)),
    onSuccess: (id, { machine, view }) => {
      addTile(queries, machine, view.id, {
        kind: "monitor",
        id,
        checkout: view.checkout ?? null,
      });
      placeTile(view, "monitor", id);
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not open the monitor",
        description: error.message,
      });
    },
    onSettled: (_, __, { machine }) =>
      queries.invalidateQueries(workspacesQuery(machine)),
  });
}

export function useShowMonitor(groups: Groups, workspace: View | undefined) {
  const open = useOpenMonitor();

  return (machine: Machine) => {
    const view =
      workspace?.machine === machine.id ? workspace : first(groups, machine);
    if (!view) {
      return;
    }

    selectWorkspace(view);
    open.mutate({ machine, view });
  };
}

function first(groups: Groups, machine: Machine): View | undefined {
  const workspace = groups.find((group) => group.machine.id === machine.id)
    ?.workspaces.data?.[0];

  return workspace ? { machine: machine.id, id: workspace.id } : undefined;
}
