import { PlusCircleIcon } from "@phosphor-icons/react";

import { Keys } from "@/components/keys";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { closeStart } from "@/features/workspaces/lib/opening";
import type { View } from "@/features/workspaces/lib/selection";
import { shortcuts } from "@/lib/shortcuts";

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
              className="h-8"
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
            <ContextMenuShortcut>
              <Keys letter={shortcuts.close} />
            </ContextMenuShortcut>
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </SidebarMenuSubItem>
  );
}
