import type { DockviewPanelApi } from "dockview-react";
import { useContext } from "react";

import { CloseTerminal } from "@/features/workspaces/components/close-terminal";
import { TerminalIcon } from "@/features/workspaces/components/terminal-icon";
import { TileBar } from "@/features/workspaces/components/tile-bar";
import { useClose } from "@/features/workspaces/hooks/use-close";
import { useTiles } from "@/features/workspaces/lib/layouts";
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
  const close = useClose(machine, terminal);
  const active = useTiles(
    (tiles) =>
      tiles.active?.machine === machine.id && tiles.active.id === terminal.id,
  );

  return (
    <section
      aria-label={terminal.name}
      className="flex h-full min-h-0 flex-col"
    >
      <TileBar
        item={{ machine: machine.id, id: terminal.id }}
        active={active}
        icon={<TerminalIcon agent={terminal.agent} />}
        title={terminal.name}
        detail={terminal.folder}
        branch={terminal.branch}
        onClose={close.closable ? close.request : undefined}
      />
      {render({ panel, machine, id: terminal.id })}
      <CloseTerminal
        machine={machine}
        terminal={terminal}
        open={close.confirming}
        onOpenChange={close.setConfirming}
      />
    </section>
  );
}
