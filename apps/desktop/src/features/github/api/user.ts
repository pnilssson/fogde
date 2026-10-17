import { queryOptions } from "@tanstack/react-query";

import { call } from "@/lib/host";

export const userQuery = queryOptions({
  queryKey: ["github.user"],
  queryFn: () => call("github.user"),
});
