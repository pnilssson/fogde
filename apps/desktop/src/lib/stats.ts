import { queryOptions } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";

export const statsQuery = queryOptions({
  queryKey: ["stats.get"],
  queryFn: () => callOwn("stats.get"),
  staleTime: Infinity,
});
