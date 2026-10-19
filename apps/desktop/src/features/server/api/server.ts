import { queryOptions } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";

export const sshHostsQuery = queryOptions({
  queryKey: ["server.hosts"],
  queryFn: () => callOwn("server.hosts"),
});
