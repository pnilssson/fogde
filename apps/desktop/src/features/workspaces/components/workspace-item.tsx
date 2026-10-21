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
import { RenameWorkspace } from "@/features/workspaces/components/rename-workspace";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { WorkspaceTree } from "@/features/workspaces/components/workspace-tree";
import { setOpen, useOpen } from "@/features/workspaces/lib/layouts";
import type { Selection } from "@/features/workspaces/lib/selection";
import { attention } from "@/features/workspaces/lib/status";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  current: boolean;
  down: boolean;
  onSelect: (workspace: Selection, terminal?: Selection) => void;
  onStart: (workspace: Selection, folder: string) => void;
}

export function WorkspaceItem({
  machine,
  workspace,
  current,
  down,
  onSelect,
  onStart,
}: Props) {
  const place = { machine: machine.id, id: workspace.id };
  const empty =
    workspace.terminals.length === 0 && workspace.checkouts.length < 2;
  const open = useOpen(place) && !empty;
  const [renaming, setRenaming] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const start = () => {
    onStart(place, workspace.folder);
  };

  const rename = () => {
    setRenaming(true);
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
                aria-expanded={empty ? undefined : open}
                onClick={() => {
                  if (current && !empty) {
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
                empty && "invisible",
                open && "rotate-90",
              )}
            />
            <FolderSimpleIcon />
            <span className="min-w-0 flex-1 truncate">{workspace.name}</span>
            {!open && !down && (
              <StateIcon
                status={attention(workspace.terminals)}
                className="group-focus-within/menu-item:invisible group-hover/menu-item:invisible"
              />
            )}
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem onClick={start}>New tile</ContextMenuItem>
            <ContextMenuItem onClick={rename}>Rename</ContextMenuItem>
            <ContextMenuSeparator />
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
              <DropdownMenuItem onClick={start}>New tile</DropdownMenuItem>
              <DropdownMenuItem onClick={rename}>Rename</DropdownMenuItem>
              <DropdownMenuSeparator />
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
            onSelect={(selection) => {
              onSelect(place, selection);
            }}
            onStart={(folder) => {
              onStart(place, folder);
            }}
          />
        </CollapsibleContent>
      </Collapsible>
      <RenameWorkspace
        machine={machine}
        workspace={workspace}
        open={renaming}
        onOpenChange={setRenaming}
      />
      <DeleteWorkspace
        machine={machine}
        workspace={workspace}
        open={deleting}
        onOpenChange={setDeleting}
      />
    </SidebarMenuItem>
  );
}
