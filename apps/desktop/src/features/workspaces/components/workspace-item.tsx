import {
  CaretRightIcon,
  DotsThreeIcon,
  FolderSimpleIcon,
} from "@phosphor-icons/react";
import { cn } from "cn";
import { useState } from "react";

import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { DeleteWorkspace } from "@/features/workspaces/components/delete-workspace";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { WorkspaceTree } from "@/features/workspaces/components/workspace-tree";
import { setOpen, useOpen } from "@/features/workspaces/lib/layouts";
import type { Selection, View } from "@/features/workspaces/lib/selection";
import { attention } from "@/features/workspaces/lib/status";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  view: View | undefined;
  down: boolean;
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
}

export function WorkspaceItem({
  machine,
  workspace,
  view,
  down,
  onSelect,
  onStart,
}: Props) {
  const place = { machine: machine.id, id: workspace.id };
  const current = view !== undefined && view.checkout === undefined;
  const open = useOpen(place);
  const [deleting, setDeleting] = useState(false);

  const start = () => {
    onStart(place, workspace.folder);
  };

  const remove = () => {
    setDeleting(true);
  };

  return (
    <SidebarMenuItem>
      <Collapsible open={open}>
        <ContextMenu disabled={down}>
          <ContextMenuTrigger
            render={
              <SidebarMenuButton
                isActive={current}
                aria-expanded={open}
                onClick={() => {
                  if (current) {
                    setOpen(place, !open);

                    return;
                  }

                  onSelect(place);
                }}
              />
            }
          >
            <CaretRightIcon
              className={cn(
                "transition-transform ease-out motion-reduce:transition-none",
                open && "rotate-90",
              )}
            />
            <FolderSimpleIcon />
            <span
              className={cn(
                "min-w-0 flex-1 truncate",
                workspace.missing && "text-muted-foreground",
              )}
            >
              {workspace.name}
            </span>
            {!open && !down && (
              <StateIcon
                status={attention(workspace.terminals)}
                className="group-focus-within/menu-item:invisible group-hover/menu-item:invisible"
              />
            )}
          </ContextMenuTrigger>
          <ContextMenuContent>
            {!workspace.missing && (
              <>
                <ContextMenuItem onClick={start}>New tile</ContextMenuItem>
                <ContextMenuSeparator />
              </>
            )}
            <ContextMenuItem variant="destructive" onClick={remove}>
              Delete
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
        {!down && (
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <SidebarMenuAction
                  showOnHover
                  aria-label={`Actions for ${workspace.name}`}
                />
              }
            >
              <DotsThreeIcon />
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {!workspace.missing && (
                <>
                  <DropdownMenuItem onClick={start}>New tile</DropdownMenuItem>
                  <DropdownMenuSeparator />
                </>
              )}
              <DropdownMenuItem variant="destructive" onClick={remove}>
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
        <CollapsibleContent>
          <WorkspaceTree
            machine={machine}
            workspace={workspace}
            view={view}
            down={down}
            onSelect={onSelect}
            onStart={onStart}
          />
        </CollapsibleContent>
      </Collapsible>
      <DeleteWorkspace
        machine={machine}
        workspace={workspace}
        open={deleting}
        onOpenChange={setDeleting}
      />
    </SidebarMenuItem>
  );
}
