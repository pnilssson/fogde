import { useQueries, useQuery, useQueryClient } from "@tanstack/react-query";

import { usageQuery } from "@/features/usage/api/usage";
import type { Usage } from "@/generated/protocol";
import { useMachineEvent } from "@/hooks/use-machine-event";
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

  useMachineEvent(machines, "usage.changed", (machine, usage) => {
    queries.setQueryData(usageQuery(machine).queryKey, usage);
  });

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
