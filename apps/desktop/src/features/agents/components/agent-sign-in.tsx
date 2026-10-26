import { AgentIcon } from "@/components/agent-icon";
import { useMutation, useQueryClient } from "@tanstack/react-query";

import { Connection } from "@/components/connection";
import { toast } from "@/components/ui/toast";
import type { Agent, AgentState } from "@/generated/protocol";
import { agents, agentsQuery } from "@/lib/agents";
import { statusOf } from "@/lib/connection";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  agent: Agent;
  state: AgentState;
  signingIn: boolean;
  onSignIn: () => void;
  onCancel?: () => void;
}

interface Mode {
  action: string;
  pending: string;
  busy: boolean;
  run: () => void;
}

export function AgentSignIn({
  machine,
  agent,
  state,
  signingIn,
  onSignIn,
  onCancel,
}: Props) {
  const name = agents[agent].account;
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

  const modes: Record<"install" | "connect", Mode> = {
    install: {
      action: "Install",
      pending: "Installing…",
      busy: install.isPending,
      run: () => {
        install.mutate();
      },
    },
    connect: {
      action: "Connect",
      pending: "Connecting…",
      busy: signingIn,
      run: onSignIn,
    },
  };
  const mode = modes[state === "missing" ? "install" : "connect"];

  return (
    <Connection
      name={name}
      icon={<AgentIcon agent={agent} />}
      action={mode.action}
      pending={mode.pending}
      status={statusOf(state === "signedIn", mode.busy)}
      onConnect={mode.run}
      onCancel={onCancel}
    />
  );
}
