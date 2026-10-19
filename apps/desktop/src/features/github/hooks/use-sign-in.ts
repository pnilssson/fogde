import { useMutation, useQueryClient } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";

import { accountsQuery } from "@/features/github/api/accounts";
import type { Machine } from "@/lib/machines";

export function useSignIn(machine: Machine) {
  const queries = useQueryClient();

  const finish = useMutation({
    mutationFn: () => machine.client.call("github.finishSignIn"),
    onSuccess: () => queries.invalidateQueries(accountsQuery(machine)),
  });
  const start = useMutation({
    mutationFn: (host: string) =>
      machine.client.call("github.startSignIn", host),
    onSuccess: (prompt) => {
      void openUrl(prompt.url);
      finish.mutate();
    },
  });

  return {
    prompt: finish.isPending ? start.data : undefined,
    starting: start.isPending ? start.variables : undefined,
    error: start.error ?? finish.error,
    begin: (host: string) => {
      finish.reset();
      start.mutate(host);
    },
  };
}
