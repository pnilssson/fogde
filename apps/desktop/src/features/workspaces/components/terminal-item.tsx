import { DotsThreeIcon } from "@phosphor-icons/react";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenuAction,
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
  onSelect: (selection: Selection) => void;
}

export function TerminalItem({ machine, terminal, onSelect }: Props) {
  const selection = { machine: machine.id, id: terminal.id };
  const close = useClose(machine, terminal);

  return (
    <SidebarMenuSubItem>
      <ContextMenu disabled={!close.closable}>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
              size="sm"
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
          <StateIcon
            status={terminal.status}
            className={
              close.closable
                ? "group-focus-within/menu-sub-item:invisible group-hover/menu-sub-item:invisible"
                : undefined
            }
          />
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onClick={close.request}>Close</ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      {close.closable && (
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <SidebarMenuAction
                aria-label={`Actions for ${terminal.name}`}
                className="inset-y-0 my-auto h-5 opacity-0 group-focus-within/menu-sub-item:opacity-100 group-hover/menu-sub-item:opacity-100 aria-expanded:opacity-100"
              />
            }
          >
            <DotsThreeIcon />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onClick={close.request}>Close</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
      <CloseTerminal
        machine={machine}
        terminal={terminal}
        open={close.confirming}
        onOpenChange={close.setConfirming}
      />
    </SidebarMenuSubItem>
  );
}
