import { queryOptions } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";

export const shownQuery = queryOptions({
  queryKey: ["usage.getShown"],
  queryFn: () => callOwn("usage.getShown"),
  staleTime: Infinity,
});
