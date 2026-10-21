import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { toast } from "@/components/ui/toast";
import { workspacesQuery } from "@/features/terminals/api/workspaces";
import { agentName } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

export function useFailures(machines: Machine[]) {
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.listen("terminals.failed", (failed) => {
        void queries.invalidateQueries(workspacesQuery(machine));
        toast.add({
          type: "error",
          title: `Could not start ${agentName[failed.agent]}`,
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
