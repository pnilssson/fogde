import { queryOptions } from "@tanstack/react-query";

import { call } from "@/lib/host";

export const accountsQuery = queryOptions({
  queryKey: ["github.accounts"],
  queryFn: () => call("github.accounts"),
});
