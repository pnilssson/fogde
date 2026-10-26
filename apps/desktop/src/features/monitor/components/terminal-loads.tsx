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
    <section aria-labelledby={heading}>
      <table className="w-full type-body">
        <thead>
          <tr className="type-caption text-muted-foreground">
            <th className="w-full pb-2 text-left font-normal">
              <h2 id={heading} className="type-title text-foreground">
                Terminals
              </h2>
            </th>
            <th className="pb-2 pl-4 text-left font-normal">
              <span className="sr-only">CPU history</span>
            </th>
            <th className="pb-2 pl-4 text-right font-normal whitespace-nowrap">
              CPU
            </th>
            <th className="pb-2 pl-4 text-right font-normal whitespace-nowrap">
              Memory
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map(({ terminal, workspace, load }) => (
            <tr key={terminal.id}>
              <td className="py-1">
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
              </td>
              <td className="py-1 pl-4">
                <span className="flex text-muted-foreground">
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
              </td>
              <td className="py-1 pl-4 text-right font-mono whitespace-nowrap tabular-nums">
                {percent(load.cpu)}
              </td>
              <td className="py-1 pl-4 text-right font-mono whitespace-nowrap tabular-nums">
                {bytes(load.memory)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
