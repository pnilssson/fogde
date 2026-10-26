import { cn } from "cn";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
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
import {
  dragProps,
  hover,
  isTerminal,
  useDragging,
  useOver,
} from "@/features/workspaces/lib/drag";
import { beforeOf, changes } from "@/features/workspaces/lib/order";
import type { Selection } from "@/features/workspaces/lib/selection";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  terminal: Terminal;
  siblings: string[];
  active: boolean;
  onSelect: (selection: Selection) => void;
  onArrange: (terminal: string, before: string | null) => void;
}

export function TerminalItem({
  machine,
  terminal,
  siblings,
  active,
  onSelect,
  onArrange,
}: Props) {
  const selection = { machine: machine.id, id: terminal.id };
  const close = useClose(machine, terminal);
  const dragging = useDragging();
  const over = useOver(terminal.id);
  const index = siblings.indexOf(terminal.id);
  const moving =
    dragging &&
    isTerminal(dragging) &&
    dragging.machine === machine.id &&
    dragging.id !== terminal.id &&
    siblings.includes(dragging.id)
      ? dragging.id
      : undefined;

  return (
    <SidebarMenuSubItem
      onDragOver={(event) => {
        if (!moving) {
          return;
        }

        const box = event.currentTarget.getBoundingClientRect();
        const edge =
          event.clientY < box.top + box.height / 2 ? "above" : "below";
        const before = beforeOf(siblings, moving, terminal.id, edge);
        if (!changes(siblings, moving, before)) {
          hover(undefined);
          return;
        }

        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        hover({ id: terminal.id, edge });
      }}
      onDragLeave={(event) => {
        if (
          event.relatedTarget instanceof Node &&
          event.currentTarget.contains(event.relatedTarget)
        ) {
          return;
        }

        hover(undefined);
      }}
      onDrop={(event) => {
        if (!moving || !over) {
          return;
        }

        event.preventDefault();
        onArrange(moving, beforeOf(siblings, moving, terminal.id, over));
        hover(undefined);
      }}
    >
      {moving && over && (
        <span
          aria-hidden
          className={cn(
            "pointer-events-none absolute inset-x-0 h-0.5 rounded-full bg-sidebar-ring",
            over === "above"
              ? "-top-0.5 -translate-y-1/2"
              : "-bottom-0.5 translate-y-1/2",
          )}
        />
      )}
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
          <ContextMenuItem
            disabled={index <= 0}
            onClick={() => {
              onArrange(terminal.id, siblings[index - 1] ?? null);
            }}
          >
            Move up
          </ContextMenuItem>
          <ContextMenuItem
            disabled={index === siblings.length - 1}
            onClick={() => {
              onArrange(terminal.id, siblings[index + 2] ?? null);
            }}
          >
            Move down
          </ContextMenuItem>
          <ContextMenuSeparator />
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
