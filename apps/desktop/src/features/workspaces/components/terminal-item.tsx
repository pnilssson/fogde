import { CloseTerminal } from "@/features/workspaces/components/close-terminal";
import { TerminalIcon } from "@/features/workspaces/components/terminal-icon";
import { TileItem } from "@/features/workspaces/components/tile-item";
import { useClose } from "@/features/workspaces/hooks/use-close";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  terminal: Terminal;
  siblings: string[];
  active: boolean;
  onSelect: () => void;
  onArrange: (tile: string, before: string | null) => void;
}

export function TerminalItem({
  machine,
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
      closable={close.closable}
      onSelect={onSelect}
      onArrange={onArrange}
      onClose={close.request}
    >
      <CloseTerminal
        machine={machine}
        terminal={terminal}
        open={close.confirming}
        onOpenChange={close.setConfirming}
      />
    </TileItem>
  );
}
