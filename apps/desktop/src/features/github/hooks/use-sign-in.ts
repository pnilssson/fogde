import { useMutation, useQueryClient } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";

import { accountsQuery } from "@/features/github/api/accounts";
import type { Machine } from "@/lib/machines";
import { hostname, trusted, UntrustedLink } from "@/lib/sign-in";

export function useSignIn(machine: Machine) {
  const queries = useQueryClient();

  const finish = useMutation({
    mutationFn: () => machine.client.call("github.finishSignIn"),
    onSuccess: () => queries.invalidateQueries(accountsQuery(machine)),
  });
  const start = useMutation({
    mutationFn: async (host: string) => {
      const prompt = await machine.client.call("github.startSignIn", host);
      if (!trusted(prompt.url, [hostname(host)])) {
        void machine.client.call("github.cancelSignIn").catch(() => undefined);
        throw new UntrustedLink(prompt.url);
      }

      return prompt;
    },
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
