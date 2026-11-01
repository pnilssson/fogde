import { useQueryClient } from "@tanstack/react-query";

import { accountsQuery } from "@/features/github/api/accounts";
import type { SignInCode } from "@/generated/protocol";
import { useSignInFlow } from "@/hooks/use-sign-in-flow";
import type { Machine } from "@/lib/machines";
import { hostname } from "@/lib/sign-in";

export function useSignIn(machine: Machine) {
  const queries = useQueryClient();
  const { active, begin, cancel } = useSignInFlow<string, SignInCode>({
    start: (host) => machine.client.call("github.startSignIn", host),
    link: (host, prompt) => ({ url: prompt.url, domains: [hostname(host)] }),
    waits: () => true,
    finish: () => machine.client.call("github.finishSignIn"),
    cancel: () => machine.client.call("github.cancelSignIn"),
    refresh: () => queries.invalidateQueries(accountsQuery(machine)),
    failed: () => "Could not sign in to GitHub",
    stuck: () => "Could not cancel signing in to GitHub",
  });

  return {
    prompt: active?.prompt,
    starting: active && !active.prompt ? active.key : undefined,
    begin,
    cancel,
  };
}
