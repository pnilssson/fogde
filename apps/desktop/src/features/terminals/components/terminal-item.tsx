import { DotsThreeIcon, TerminalWindowIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { AgentIcon } from "@/components/agent-icon";
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
import { CloseTerminal } from "@/features/terminals/components/close-terminal";
import { StateIcon } from "@/features/terminals/components/state-icon";
import {
  useCloseTerminal,
  useClosing,
  useConnection,
} from "@/features/terminals/hooks/use-terminals";
import type { Selection } from "@/features/terminals/lib/selection";
import { dragType, endDrag, startDrag } from "@/features/terminals/lib/drag";
import { busy } from "@/features/terminals/lib/status";
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
  const close = useCloseTerminal(machine);
  const connection = useConnection(machine);
  const pending = useClosing(machine, terminal);
  const closable = connection.state === "connected" && !pending;
  const [confirming, setConfirming] = useState(false);

  const closing = () => {
    if (busy(terminal.status)) {
      setConfirming(true);
      return;
    }

    close.mutate(terminal);
  };

  return (
    <SidebarMenuSubItem>
      <ContextMenu disabled={!closable}>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
              isActive={active}
              render={<button type="button" />}
              draggable
              onDragStart={(event) => {
                event.dataTransfer.setData(dragType, JSON.stringify(selection));
                startDrag(selection);
              }}
              onDragEnd={endDrag}
              onClick={() => {
                onSelect(selection);
              }}
            />
          }
        >
          {terminal.agent ? (
            <AgentIcon agent={terminal.agent} />
          ) : (
            <TerminalWindowIcon />
          )}
          <span className="min-w-0 flex-1 truncate">{terminal.name}</span>
          <StateIcon
            status={terminal.status}
            className={
              closable
                ? "group-focus-within/menu-sub-item:invisible group-hover/menu-sub-item:invisible"
                : undefined
            }
          />
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onClick={closing}>Close</ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      {closable && (
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
            <DropdownMenuItem onClick={closing}>Close</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
      <CloseTerminal
        machine={machine}
        terminal={terminal}
        open={confirming}
        onOpenChange={setConfirming}
      />
    </SidebarMenuSubItem>
  );
}
