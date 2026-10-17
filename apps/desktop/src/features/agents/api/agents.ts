import { queryOptions } from "@tanstack/react-query";

import { call } from "@/lib/host";

export const signedInQuery = queryOptions({
  queryKey: ["agents.signedIn"],
  queryFn: () => call("agents.signedIn"),
});
