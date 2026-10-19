import { XIcon } from "@phosphor-icons/react";
import type { DockviewPanelApi } from "dockview-react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { TerminalView } from "@/features/terminals/components/terminal-view";
import {
  useCloseTerminal,
  useConnection,
} from "@/features/terminals/hooks/use-terminals";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  panel: DockviewPanelApi;
  machine: Machine;
  terminal: Terminal;
}

export function TerminalPane({ panel, machine, terminal }: Props) {
  const close = useCloseTerminal(machine);
  const connection = useConnection(machine);

  return (
    <section
      aria-label={terminal.name}
      className="relative flex h-full min-h-0 flex-col overflow-hidden rounded-xl bg-surface-3 shadow-surface-3"
    >
      <header className="flex h-8 shrink-0 items-center gap-2 px-2">
        <span className="min-w-0 flex-1 truncate type-caption">
          {terminal.name}
        </span>
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="sm"
                aria-label={`Close ${terminal.name}`}
                disabled={close.isPending || connection.state !== "connected"}
                aria-busy={close.isPending}
                onClick={() => {
                  close.mutate(terminal);
                }}
              />
            }
          >
            {close.isPending ? <Spinner aria-hidden /> : <XIcon />}
          </TooltipTrigger>
          <TooltipContent>Close terminal</TooltipContent>
        </Tooltip>
      </header>
      <TerminalView panel={panel} machine={machine} id={terminal.id} />
    </section>
  );
}
