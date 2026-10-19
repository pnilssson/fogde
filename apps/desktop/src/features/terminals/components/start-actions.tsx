import { TerminalWindowIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { AgentIcon } from "@/components/agent-icon";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useCreateTerminal } from "@/features/terminals/hooks/use-terminals";
import { opened } from "@/features/terminals/lib/tiles";
import type { Agent } from "@/generated/protocol";
import { agentName, agentsQuery } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: string;
  folder: string;
}

export function StartActions({ machine, workspace, folder }: Props) {
  const { data: agents = [] } = useQuery(agentsQuery(machine));
  const create = useCreateTerminal(machine, (terminal) => {
    opened({ machine: machine.id, id: terminal.id });
  });
  const choices: (Agent | null)[] = [
    null,
    ...agents
      .filter(({ state }) => state === "signedIn")
      .map(({ agent }) => agent),
  ];

  return (
    <div className="flex flex-wrap justify-center gap-2">
      {choices.map((agent) => {
        const opening = create.isPending && create.variables.agent === agent;
        const name = agent ? agentName[agent] : "Terminal";

        return (
          <Button
            key={agent ?? "terminal"}
            variant="secondary"
            size="sm"
            disabled={create.isPending}
            aria-busy={opening}
            onClick={() => {
              create.mutate({ workspace, folder, agent });
            }}
          >
            {opening ? (
              <Spinner aria-hidden data-icon="inline-start" />
            ) : agent ? (
              <AgentIcon agent={agent} />
            ) : (
              <TerminalWindowIcon data-icon="inline-start" />
            )}
            {opening ? "Opening…" : name}
          </Button>
        );
      })}
    </div>
  );
}
