import { cn } from "cn";
import type { ReactNode } from "react";

import { Keys } from "@/components/keys";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import {
  dragProps,
  hover,
  isListed,
  useDragging,
  useOver,
} from "@/features/workspaces/lib/drag";
import { beforeOf, changes } from "@/features/workspaces/lib/order";
import type { Selection } from "@/features/workspaces/lib/selection";
import type { Status } from "@/generated/protocol";
import { shortcuts } from "@/lib/shortcuts";

interface Props {
  tile: Selection;
  siblings: string[];
  active: boolean;
  icon: ReactNode;
  label: string;
  status?: Status | undefined;
  closable: boolean;
  onSelect: () => void;
  onArrange: (tile: string, before: string | null) => void;
  onClose: () => void;
  children?: ReactNode;
}

export function TileItem({
  tile,
  siblings,
  active,
  icon,
  label,
  status,
  closable,
  onSelect,
  onArrange,
  onClose,
  children,
}: Props) {
  const dragging = useDragging();
  const over = useOver(tile.id);
  const index = siblings.indexOf(tile.id);
  const moving =
    dragging &&
    isListed(dragging) &&
    dragging.machine === tile.machine &&
    dragging.id !== tile.id &&
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
        const before = beforeOf(siblings, moving, tile.id, edge);
        if (!changes(siblings, moving, before)) {
          hover(undefined);
          return;
        }

        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        hover({ id: tile.id, edge });
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
        onArrange(moving, beforeOf(siblings, moving, tile.id, over));
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
      <ContextMenu>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
              size="sm"
              className="h-8"
              isActive={active}
              render={<button type="button" />}
              {...dragProps(tile)}
              onClick={onSelect}
            />
          }
        >
          {icon}
          <span className="min-w-0 flex-1 truncate">{label}</span>
          {status && <StateIcon status={status} />}
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem
            disabled={index <= 0}
            onClick={() => {
              onArrange(tile.id, siblings[index - 1] ?? null);
            }}
          >
            Move up
          </ContextMenuItem>
          <ContextMenuItem
            disabled={index === siblings.length - 1}
            onClick={() => {
              onArrange(tile.id, siblings[index + 2] ?? null);
            }}
          >
            Move down
          </ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem disabled={!closable} onClick={onClose}>
            Close
            <ContextMenuShortcut>
              <Keys letter={shortcuts.close} />
            </ContextMenuShortcut>
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      {children}
    </SidebarMenuSubItem>
  );
}
