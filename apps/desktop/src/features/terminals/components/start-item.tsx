import { DotsThreeIcon, PlusCircleIcon } from "@phosphor-icons/react";

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
import { closeStart } from "@/features/terminals/lib/tiles";

interface Props {
  active: boolean;
  onSelect: () => void;
}

export function StartItem({ active, onSelect }: Props) {
  return (
    <SidebarMenuSubItem>
      <ContextMenu>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
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
          <ContextMenuItem onClick={closeStart}>Close</ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <SidebarMenuAction
              aria-label="Actions for New tile"
              className="inset-y-0 my-auto h-5 opacity-0 group-focus-within/menu-sub-item:opacity-100 group-hover/menu-sub-item:opacity-100 aria-expanded:opacity-100"
            />
          }
        >
          <DotsThreeIcon />
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuItem onClick={closeStart}>Close</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </SidebarMenuSubItem>
  );
}
