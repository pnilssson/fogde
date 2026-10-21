import { TerminalWindowIcon } from "@phosphor-icons/react";
import { useEffect, useRef } from "react";

import { AgentIcon } from "@/components/agent-icon";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import {
  useChoices,
  useCreateTerminal,
} from "@/features/workspaces/hooks/use-terminals";
import {
  startFocused,
  useStartFocus,
} from "@/features/workspaces/lib/start-focus";
import { agentName } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: string;
  folder: string;
}

export function StartActions({ machine, workspace, folder }: Props) {
  const choices = useChoices(machine);
  const create = useCreateTerminal();
  const focus = useStartFocus();
  const first = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (focus && first.current) {
      first.current.focus();
      startFocused();
    }
  }, [focus]);

  return (
    <div className="flex flex-wrap justify-center gap-2">
      {choices.map((agent) => {
        const opening = create.isPending && create.variables.agent === agent;
        const name = agent ? agentName[agent] : "Terminal";

        return (
          <Button
            key={agent ?? "terminal"}
            ref={agent === choices[0] ? first : undefined}
            variant="secondary"
            size="sm"
            disabled={create.isPending}
            aria-busy={opening}
            onClick={() => {
              create.mutate({ machine, workspace, folder, agent });
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
