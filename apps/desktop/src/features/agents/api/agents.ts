import { queryOptions } from "@tanstack/react-query";

import { call } from "@/lib/host";

export const agentsQuery = queryOptions({
  queryKey: ["agents.list"],
  queryFn: () => call("agents.list"),
});
