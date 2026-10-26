import { queryOptions } from "@tanstack/react-query";

import type { Machine } from "@/lib/machines";

export { agents, allAgents } from "@/generated/protocol";

export const agentsQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "agents.list"],
    queryFn: () => machine.client.call("agents.list"),
  });
