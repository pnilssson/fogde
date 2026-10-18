import { useMutation, useQueryClient } from "@tanstack/react-query";

import { Connection } from "@/components/connection";
import { toast } from "@/components/ui/toast";
import { agentsQuery } from "@/features/agents/api/agents";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import type { Agent, AgentState } from "@/generated/protocol";
import { call } from "@/lib/host";

const accounts: Record<Agent, string> = {
  claude: "Claude",
  codex: "ChatGPT",
  copilot: "GitHub Copilot",
};

interface Props {
  agent: Agent;
  state: AgentState;
}

export function AgentSignIn({ agent, state }: Props) {
  const name = accounts[agent];
  const queries = useQueryClient();
  const refresh = () => queries.invalidateQueries(agentsQuery);
  const fail = (title: string) => (error: Error) => {
    toast.add({ type: "error", title, description: error.message });
  };

  const install = useMutation({
    mutationFn: () => call("agents.install", agent),
    onSuccess: refresh,
    onError: fail(`Could not install ${name}`),
  });
  const signIn = useMutation({
    mutationFn: () => call("agents.signIn", agent),
    onSuccess: refresh,
    onError: fail(`Could not connect ${name}`),
  });
  const action = state === "missing" ? install : signIn;

  return (
    <Connection
      name={name}
      icon={<AgentIcon agent={agent} />}
      action={state === "missing" ? "Install" : "Connect"}
      pending={state === "missing" ? "Installing…" : "Connect"}
      status={
        state === "signedIn"
          ? "connected"
          : action.isPending
            ? "connecting"
            : "disconnected"
      }
      onConnect={() => {
        action.mutate();
      }}
    />
  );
}
