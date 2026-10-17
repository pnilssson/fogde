import { useMutation, useQueryClient } from "@tanstack/react-query";

import { Connection } from "@/components/connection";
import { signedInQuery } from "@/features/agents/api/agents";
import type { Agent } from "@/generated/protocol";
import { call } from "@/lib/host";

const accounts: Record<Agent, string> = {
  claude: "Claude",
  codex: "ChatGPT",
};

interface Props {
  agent: Agent;
  signedIn: boolean;
}

export function AgentSignIn({ agent, signedIn }: Props) {
  const queries = useQueryClient();
  const signIn = useMutation({
    mutationFn: () => call("agents.signIn", agent),
    onSuccess: () => queries.invalidateQueries(signedInQuery),
  });

  return (
    <div className="flex flex-col gap-2">
      <Connection
        name={accounts[agent]}
        status={
          signedIn
            ? "connected"
            : signIn.isPending
              ? "connecting"
              : "disconnected"
        }
        onConnect={() => {
          signIn.mutate();
        }}
      />
      {signIn.error && !signedIn && (
        <p className="type-body">{signIn.error.message}</p>
      )}
    </div>
  );
}
