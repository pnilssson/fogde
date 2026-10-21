import type { DockviewPanelApi } from "dockview-react";

import { TerminalView } from "@/features/terminals/components/terminal-view";
import { TileGrip } from "@/features/terminals/components/tile-grip";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  panel: DockviewPanelApi;
  machine: Machine;
  terminal: Terminal;
}

export function TerminalPane({ panel, machine, terminal }: Props) {
  return (
    <section
      aria-label={terminal.name}
      className="group/tile relative flex h-full min-h-0 flex-col overflow-hidden rounded-xl bg-surface-3 shadow-surface-3"
    >
      <TileGrip item={{ machine: machine.id, id: terminal.id }} />
      <TerminalView panel={panel} machine={machine} id={terminal.id} />
    </section>
  );
}
