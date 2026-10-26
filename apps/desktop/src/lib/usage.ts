import { queryOptions } from "@tanstack/react-query";

import type { Agent, Shown } from "@/generated/protocol";
import { callOwn } from "@/lib/host";

export const shownQuery = queryOptions({
  queryKey: ["usage.getShown"],
  queryFn: () => callOwn("usage.getShown"),
  staleTime: Infinity,
});

export function shows(shown: Shown, agent: Agent) {
  return shown[agent] ?? true;
}
