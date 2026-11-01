import type { ReactNode } from "react";

import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import {
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { DropLine } from "@/features/workspaces/components/drop-line";
import { RowContext } from "@/features/workspaces/components/row-menu";
import { Since } from "@/features/workspaces/components/since";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { useReorder } from "@/features/workspaces/hooks/use-reorder";
import {
  dragProps,
  isListed,
  useDragging,
} from "@/features/workspaces/lib/drag";
import { moves } from "@/features/workspaces/lib/order";
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
  since?: number | undefined;
  closable: boolean;
  onSelect: () => void;
  onArrange: (tile: string, before: string | null) => void;
  onClose: () => void;
}

export function TileItem({
  tile,
  siblings,
  active,
  icon,
  label,
  status,
  since,
  closable,
  onSelect,
  onArrange,
  onClose,
}: Props) {
  const dragging = useDragging();
  const moving =
    dragging &&
    isListed(dragging) &&
    dragging.machine === tile.machine &&
    dragging.id !== tile.id &&
    siblings.includes(dragging.id)
      ? dragging.id
      : undefined;
  const drop = useReorder(tile.id, siblings, moving, onArrange);

  return (
    <SidebarMenuSubItem {...drop.target}>
      <DropLine edge={drop.over} />
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
          {since !== undefined && status !== "working" && <Since at={since} />}
          {status && <StateIcon status={status} />}
        </ContextMenuTrigger>
        <RowContext
          groups={[
            moves(tile.id, siblings, onArrange),
            [
              {
                label: "Close",
                disabled: !closable,
                shortcut: shortcuts.close,
                onSelect: onClose,
              },
            ],
          ]}
        />
      </ContextMenu>
    </SidebarMenuSubItem>
  );
}
