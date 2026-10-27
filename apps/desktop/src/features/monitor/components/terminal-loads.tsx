import { TerminalWindowIcon } from "@phosphor-icons/react";
import { useId } from "react";

import { AgentIcon } from "@/components/agent-icon";
import { Bars } from "@/features/monitor/components/sparkline";
import type { Reading } from "@/features/monitor/hooks/use-samples";
import { bytes, percent } from "@/features/monitor/lib/format";
import type { Workspace } from "@/generated/protocol";

const HISTORY = 30;

interface Props {
  samples: Reading[];
  workspaces: Workspace[];
}

export function TerminalLoads({ samples, workspaces }: Props) {
  const heading = useId();
  const latest = samples.at(-1)?.terminals ?? [];
  const rows = workspaces.flatMap((workspace) =>
    workspace.tiles.flatMap((terminal) => {
      const load =
        terminal.kind === "terminal"
          ? latest.find(({ id }) => id === terminal.id)
          : undefined;

      return terminal.kind === "terminal" && load
        ? [{ terminal, workspace: workspace.name, load }]
        : [];
    }),
  );
  if (rows.length === 0) {
    return null;
  }

  const recent = samples.slice(-HISTORY);

  return (
    <section
      className="grid grid-cols-[minmax(0,1fr)_auto_auto] gap-x-4 type-body @sm:grid-cols-[minmax(0,1fr)_auto_auto_auto]"
      aria-labelledby={heading}
    >
      <div className="col-span-full grid grid-cols-subgrid items-baseline pb-2">
        <h2 id={heading} className="type-title">
          Terminals
        </h2>
        <span className="hidden @sm:block" />
        <span className="text-right type-caption text-muted-foreground">
          CPU
        </span>
        <span className="text-right type-caption text-muted-foreground">
          Memory
        </span>
      </div>
      <ul className="col-span-full grid grid-cols-subgrid">
        {rows.map(({ terminal, workspace, load }) => (
          <li
            key={terminal.id}
            className="col-span-full grid grid-cols-subgrid items-center py-1"
          >
            <div className="flex min-w-0 items-center gap-2">
              {terminal.agent ? (
                <AgentIcon agent={terminal.agent} />
              ) : (
                <TerminalWindowIcon className="size-4 shrink-0" />
              )}
              <span className="truncate">{terminal.name}</span>
              <span className="truncate text-muted-foreground">
                in {workspace}
              </span>
            </div>
            <span className="hidden text-muted-foreground @sm:flex">
              <Bars
                values={recent.map(
                  (reading) =>
                    reading.terminals.find(({ id }) => id === terminal.id)
                      ?.cpu ?? 0,
                )}
                width={96}
                height={16}
                domain={[0, 100]}
              />
            </span>
            <span className="text-right font-mono tabular-nums">
              {percent(load.cpu)}
            </span>
            <span className="text-right font-mono tabular-nums">
              {bytes(load.memory)}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
