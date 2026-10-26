import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { toast } from "@/components/ui/toast";
import { workspacesQuery } from "@/features/workspaces/api/workspaces";
import { agents } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

export function useFailures(machines: Machine[]) {
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.listen("terminals.failed", (failed) => {
        void queries.invalidateQueries(workspacesQuery(machine));
        toast.add({
          type: "error",
          title: `Could not start ${agents[failed.agent].name}`,
          description: failed.error,
        });
      }),
    );

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, queries]);
}
