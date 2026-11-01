import { useQueryClient } from "@tanstack/react-query";

import { workspacesQuery } from "@/features/workspaces/api/workspaces";
import { useMachineEvent } from "@/hooks/use-machine-event";
import type { Machine } from "@/lib/machines";

export function useCheckouts(machines: Machine[]) {
  const queries = useQueryClient();

  useMachineEvent(machines, "workspaces.changed", (machine) => {
    void queries.invalidateQueries(workspacesQuery(machine));
  });
}
