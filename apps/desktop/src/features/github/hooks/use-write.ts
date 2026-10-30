import { useMutation, useQueryClient } from "@tanstack/react-query";

import type { Client } from "@/lib/client";
import type { Machine } from "@/lib/machines";
import { report } from "@/lib/report";

const kept = new Set(["github.options", "github.accounts", "github.log"]);

export type Write<T> = (client: Client) => Promise<T>;

export function useWrite<T = unknown>(machine: Machine, failure: string) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: (write: Write<T>) => write(machine.client),
    onSettled: () =>
      queries.invalidateQueries({
        predicate: ({ queryKey: [id, method] }) =>
          id === machine.id &&
          typeof method === "string" &&
          (method.startsWith("github.") || method.startsWith("checkouts.")) &&
          !kept.has(method),
      }),
    onError: (error) => {
      report(failure, error);
    },
  });
}
