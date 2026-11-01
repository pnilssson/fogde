import { type QueryClient, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { toast } from "@/components/ui/toast";
import { useConnected } from "@/hooks/use-connected";
import { agentsQuery } from "@/lib/agents";
import { ConnectionError } from "@/lib/client";
import type { Machine } from "@/lib/machines";

export function useAgents(machines: Machine[]) {
  const queries = useQueryClient();

  useConnected(machines, (machine) => {
    load(queries, machine, 0);
  });

  useEffect(() => {
    for (const machine of machines) {
      if (machine.client.getSnapshot().state === "connected") {
        load(queries, machine, Infinity);
      }
    }
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
