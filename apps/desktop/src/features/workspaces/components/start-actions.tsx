import { NotebookIcon, TerminalWindowIcon } from "@phosphor-icons/react";
import { useEffect, useRef } from "react";

import { AgentIcon } from "@/components/agent-icon";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import {
  useChoices,
  useCreateTerminal,
} from "@/features/workspaces/hooks/use-terminals";
import { openNotes } from "@/features/workspaces/lib/opening";
import {
  startFocused,
  useStartFocus,
} from "@/features/workspaces/lib/start-focus";
import { agents } from "@/lib/agents";
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
  const wanted = useRef(false);
  const loaded = choices !== undefined;

  useEffect(() => {
    if (focus) {
      wanted.current = true;
    }

    if (wanted.current && first.current) {
      first.current.focus();
      wanted.current = false;
      startFocused();
    }
  }, [focus, loaded]);

  if (!choices) {
    return (
      <div className="flex h-8 items-center">
        <Spinner />
      </div>
    );
  }

  return (
    <div className="flex flex-wrap justify-center gap-2">
      {choices.map((agent) => {
        const opening = create.isPending && create.variables.agent === agent;
        const name = agent ? agents[agent].name : "Terminal";

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
      <Button
        variant="secondary"
        size="sm"
        disabled={create.isPending}
        onClick={() => {
          openNotes(machine.id);
        }}
      >
        <NotebookIcon data-icon="inline-start" />
        Notes
      </Button>
    </div>
  );
}
