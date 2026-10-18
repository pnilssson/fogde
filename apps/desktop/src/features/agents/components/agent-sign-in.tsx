import { useMutation, useQueryClient } from "@tanstack/react-query";

import { Connection } from "@/components/connection";
import { toast } from "@/components/ui/toast";
import { agentsQuery } from "@/features/agents/api/agents";
import { AgentIcon } from "@/features/agents/components/agent-icon";
import { accountName } from "@/features/agents/lib/account-name";
import type { Agent, AgentState } from "@/generated/protocol";
import { call } from "@/lib/host";

interface Props {
  agent: Agent;
  state: AgentState;
  signingIn: boolean;
  onSignIn: () => void;
  onCancel?: () => void;
}

export function AgentSignIn({
  agent,
  state,
  signingIn,
  onSignIn,
  onCancel,
}: Props) {
  const name = accountName[agent];
  const queries = useQueryClient();
  const install = useMutation({
    mutationFn: () => call("agents.install", agent),
    onSuccess: () => queries.invalidateQueries(agentsQuery),
    onError: (error) => {
      toast.add({
        type: "error",
        title: `Could not install ${name}`,
        description: error.message,
      });
    },
  });

  const missing = state === "missing";
  const busy = missing ? install.isPending : signingIn;

  return (
    <Connection
      name={name}
      icon={<AgentIcon agent={agent} />}
      action={missing ? "Install" : "Connect"}
      pending={missing ? "Installing…" : "Connecting…"}
      status={
        state === "signedIn"
          ? "connected"
          : busy
            ? "connecting"
            : "disconnected"
      }
      onConnect={() => {
        if (missing) {
          install.mutate();
        } else {
          onSignIn();
        }
      }}
      onCancel={onCancel}
    />
  );
}
