import { memo } from "react";

import { TerminalIcon } from "@/features/workspaces/components/terminal-icon";
import { TileItem } from "@/features/workspaces/components/tile-item";
import { useClose } from "@/features/workspaces/hooks/use-close";
import type { Selection, View } from "@/features/workspaces/lib/selection";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  view: View;
  terminal: Terminal;
  siblings: string[];
  active: boolean;
  onSelect: (view: View, tile?: Selection) => void;
  onArrange: (tile: string, before: string | null) => void;
}

export const TerminalItem = memo(function TerminalItem({
  machine,
  view,
  terminal,
  siblings,
  active,
  onSelect,
  onArrange,
}: Props) {
  const close = useClose(machine, terminal);

  return (
    <TileItem
      tile={{ machine: machine.id, id: terminal.id }}
      siblings={siblings}
      active={active}
      icon={<TerminalIcon agent={terminal.agent} />}
      label={terminal.name}
      status={terminal.status}
      since={terminal.active ?? undefined}
      closable={close.closable}
      onSelect={() => {
        onSelect(view, { machine: machine.id, id: terminal.id });
      }}
      onArrange={onArrange}
      onClose={close.request}
    />
  );
});
