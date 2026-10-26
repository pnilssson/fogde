import { useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { usageQuery } from "@/features/usage/api/usage";
import type { Usage } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { allAgents } from "@/lib/agents";
import { shownQuery, shows } from "@/lib/usage";

export function useUsage(machines: Machine[]) {
  const queries = useQueryClient();
  const { data: shown } = useQuery(shownQuery);
  const readings = useQueries({
    queries: machines.map(usageQuery),
    combine: (results) => results.flatMap(({ data }) => data ?? []),
  });

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.listen("usage.changed", (usage) => {
        queries.setQueryData(usageQuery(machine).queryKey, usage);
      }),
    );

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, queries]);

  if (!shown) {
    return [];
  }

  return allAgents.flatMap((agent) => {
    if (!shows(shown, agent)) {
      return [];
    }

    const newest = readings
      .filter((usage) => usage.agent === agent)
      .reduce<Usage | undefined>(
        (best, usage) => (best && best.at >= usage.at ? best : usage),
        undefined,
      );

    return newest ? [newest] : [];
  });
}
