import { type QueryClient, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { toast } from "@/components/ui/toast";
import { agentsQuery } from "@/lib/agents";
import { ConnectionError } from "@/lib/client";
import type { Machine } from "@/lib/machines";

export function useAgents(machines: Machine[]) {
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) => {
      let state = machine.client.getSnapshot().state;
      if (state === "connected") {
        load(queries, machine, Infinity);
      }

      return machine.client.subscribe(() => {
        const next = machine.client.getSnapshot().state;
        if (next === "connected" && state !== "connected") {
          load(queries, machine, 0);
        }

        state = next;
      });
    });

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, queries]);
}

function load(queries: QueryClient, machine: Machine, staleTime: number) {
  const options = { ...agentsQuery(machine), staleTime };
  const failed = queries.getQueryState(options.queryKey)?.status === "error";

  queries.query(options).catch((error: unknown) => {
    if (failed || error instanceof ConnectionError) {
      return;
    }

    toast.add({
      type: "error",
      title: "Could not load agents",
      description: error instanceof Error ? error.message : undefined,
    });
  });
}
