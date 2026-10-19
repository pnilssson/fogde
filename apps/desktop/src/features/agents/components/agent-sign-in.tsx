import { AgentIcon } from "@/components/agent-icon";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { Connection } from "@/components/connection";
import { toast } from "@/components/ui/toast";
import { accountName } from "@/features/agents/lib/account-name";
import type { Agent, AgentState } from "@/generated/protocol";
import { agentsQuery } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  agent: Agent;
  state: AgentState;
  signingIn: boolean;
  onSignIn: () => void;
  onCancel?: () => void;
}

export function AgentSignIn({
  machine,
  agent,
  state,
  signingIn,
  onSignIn,
  onCancel,
}: Props) {
  const name = accountName[agent];
  const queries = useQueryClient();
  const install = useMutation({
    mutationFn: () => machine.client.call("agents.install", agent),
    onSuccess: () => queries.invalidateQueries(agentsQuery(machine)),
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
