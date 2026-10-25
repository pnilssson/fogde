import { NotebookIcon } from "@phosphor-icons/react";

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
import { dragProps } from "@/features/workspaces/lib/drag";
import { closeTile } from "@/features/workspaces/lib/opening";
import type { View } from "@/features/workspaces/lib/selection";

interface Props {
  view: View;
  id: string;
  title: string;
  active: boolean;
  onSelect: () => void;
}

export function NoteItem({ view, id, title, active, onSelect }: Props) {
  return (
    <SidebarMenuSubItem>
      <ContextMenu>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
              size="sm"
              isActive={active}
              render={<button type="button" />}
              {...dragProps({ tile: id, view })}
              onClick={onSelect}
            />
          }
        >
          <NotebookIcon />
          <span className="min-w-0 flex-1 truncate">{title}</span>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem
            onClick={() => {
              closeTile(id, view);
            }}
          >
            Close
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </SidebarMenuSubItem>
  );
}
