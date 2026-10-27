import {
  DotsThreeIcon,
  FolderIcon,
  FolderOpenIcon,
} from "@phosphor-icons/react";
import { cn } from "cn";
import { useState } from "react";

import { Keys } from "@/components/keys";

import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { DeleteWorkspace } from "@/features/workspaces/components/delete-workspace";
import { NewWorktree } from "@/features/workspaces/components/new-worktree";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { WorkspaceTree } from "@/features/workspaces/components/workspace-tree";
import { mainCheckout } from "@/features/workspaces/lib/checkouts";
import { setOpen, useOpen } from "@/features/workspaces/lib/layouts";
import type { Selection, View } from "@/features/workspaces/lib/selection";
import { attention } from "@/features/workspaces/lib/status";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { shortcuts } from "@/lib/shortcuts";

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
  const [naming, setNaming] = useState(false);
  const repo = mainCheckout(workspace) !== undefined;

  const start = () => {
    onStart(place, workspace.folder);
  };

  const branch = () => {
    setNaming(true);
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
                isActive={view !== undefined}
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
            {open ? <FolderOpenIcon /> : <FolderIcon />}
            <span
              className={cn(
                "min-w-0 flex-1 truncate",
                workspace.missing && "text-muted-foreground",
              )}
            >
              {workspace.name}
            </span>
            {!open && !down && (
              <StateIcon status={attention(workspace.terminals)} />
            )}
          </ContextMenuTrigger>
          <ContextMenuContent>
            {!workspace.missing && (
              <>
                <ContextMenuItem onClick={start}>
                  New tile
                  <ContextMenuShortcut>
                    <Keys letter={shortcuts.tile} />
                  </ContextMenuShortcut>
                </ContextMenuItem>
                {repo && (
                  <ContextMenuItem onClick={branch}>
                    New worktree
                  </ContextMenuItem>
                )}
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
                  aria-label={`Actions for ${workspace.name}`}
                  className="pointer-events-none opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 aria-expanded:opacity-100"
                />
              }
            >
              <DotsThreeIcon />
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {!workspace.missing && (
                <>
                  <DropdownMenuItem onClick={start}>
                    New tile
                    <DropdownMenuShortcut>
                      <Keys letter={shortcuts.tile} />
                    </DropdownMenuShortcut>
                  </DropdownMenuItem>
                  {repo && (
                    <DropdownMenuItem onClick={branch}>
                      New worktree
                    </DropdownMenuItem>
                  )}
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
      <NewWorktree
        machine={machine}
        workspace={workspace}
        open={naming}
        onOpenChange={setNaming}
      />
    </SidebarMenuItem>
  );
}
