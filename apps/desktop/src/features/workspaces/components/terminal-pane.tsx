import type { DockviewPanelApi } from "dockview-react";
import { useContext } from "react";

import { TileGrip } from "@/features/workspaces/components/tile-grip";
import { TerminalSlot } from "@/features/workspaces/lib/slot";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  panel: DockviewPanelApi;
  machine: Machine;
  terminal: Terminal;
}

export function TerminalPane({ panel, machine, terminal }: Props) {
  const render = useContext(TerminalSlot);

  return (
    <section
      aria-label={terminal.name}
      className="group/tile relative flex h-full min-h-0 flex-col overflow-hidden rounded-xl bg-surface-3 shadow-surface-3"
    >
      <TileGrip item={{ machine: machine.id, id: terminal.id }} />
      {render({ panel, machine, id: terminal.id })}
    </section>
  );
}
