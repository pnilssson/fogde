import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { CloseTerminal } from "@/features/workspaces/components/close-terminal";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { TerminalIcon } from "@/features/workspaces/components/terminal-icon";
import { useClose } from "@/features/workspaces/hooks/use-close";
import { dragProps } from "@/features/workspaces/lib/drag";
import type { Selection } from "@/features/workspaces/lib/selection";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  terminal: Terminal;
  active: boolean;
  onSelect: (selection: Selection) => void;
}

export function TerminalItem({ machine, terminal, active, onSelect }: Props) {
  const selection = { machine: machine.id, id: terminal.id };
  const close = useClose(machine, terminal);

  return (
    <SidebarMenuSubItem>
      <ContextMenu disabled={!close.closable}>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
              size="sm"
              isActive={active}
              render={<button type="button" />}
              {...dragProps(selection)}
              onClick={() => {
                onSelect(selection);
              }}
            />
          }
        >
          <TerminalIcon agent={terminal.agent} />
          <span className="min-w-0 flex-1 truncate">{terminal.name}</span>
          <StateIcon status={terminal.status} />
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onClick={close.request}>Close</ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      <CloseTerminal
        machine={machine}
        terminal={terminal}
        open={close.confirming}
        onOpenChange={close.setConfirming}
      />
    </SidebarMenuSubItem>
  );
}
