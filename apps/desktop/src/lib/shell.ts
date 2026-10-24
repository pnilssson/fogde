import { queryOptions } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";

export const shellQuery = queryOptions({
  queryKey: ["shell.get"],
  queryFn: () => callOwn("shell.get"),
});

export const shellsQuery = queryOptions({
  queryKey: ["shell.list"],
  queryFn: () => callOwn("shell.list"),
});
