import { PlusCircleIcon } from "@phosphor-icons/react";

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
import { closeStart } from "@/features/workspaces/lib/opening";
import type { View } from "@/features/workspaces/lib/selection";

interface Props {
  view: View;
  active: boolean;
  onSelect: () => void;
}

export function StartItem({ view, active, onSelect }: Props) {
  return (
    <SidebarMenuSubItem>
      <ContextMenu>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
              size="sm"
              isActive={active}
              render={<button type="button" />}
              onClick={onSelect}
            />
          }
        >
          <PlusCircleIcon />
          <span>New tile</span>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem
            onClick={() => {
              closeStart(view);
            }}
          >
            Close
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </SidebarMenuSubItem>
  );
}
