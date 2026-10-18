import { queryOptions } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";

export const serverQuery = queryOptions({
  queryKey: ["server.get"],
  queryFn: () => callOwn("server.get"),
  meta: { own: true },
});

export const sshHostsQuery = queryOptions({
  queryKey: ["server.hosts"],
  queryFn: () => callOwn("server.hosts"),
  meta: { own: true },
});
