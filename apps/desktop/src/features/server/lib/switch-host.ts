import type { QueryClient } from "@tanstack/react-query";

import { reconnect } from "@/lib/host";

export function switchHost(queries: QueryClient) {
  reconnect();
  queries.removeQueries({ predicate: (query) => query.meta?.own !== true });
}
