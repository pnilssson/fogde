import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { workspacesQuery } from "@/features/workspaces/api/workspaces";
import type { Machine } from "@/lib/machines";

export function useCheckouts(machines: Machine[]) {
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.listen("workspaces.changed", () => {
        void queries.invalidateQueries(workspacesQuery(machine));
      }),
    );

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, queries]);
}
