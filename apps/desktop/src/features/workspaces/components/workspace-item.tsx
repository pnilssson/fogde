import { DotsThreeIcon } from "@phosphor-icons/react";
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
import { BranchStats } from "@/features/workspaces/components/branch-stats";
import { DeleteWorkspace } from "@/features/workspaces/components/delete-workspace";
import { NewWorktree } from "@/features/workspaces/components/new-worktree";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { WorkspaceTree } from "@/features/workspaces/components/workspace-tree";
import { changesOf, mainCheckout } from "@/features/workspaces/lib/checkouts";
import { setOpen, useOpen } from "@/features/workspaces/lib/layouts";
import { beforeOf, changes } from "@/features/workspaces/lib/order";
import {
  hoverWorkspace,
  reorderProps,
  useMoving,
  useOverWorkspace,
} from "@/features/workspaces/lib/reorder";
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
  siblings: string[];
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
  onArrange: (workspace: string, before: string | null) => void;
}

export function WorkspaceItem({
  machine,
  workspace,
  view,
  down,
  siblings,
  onSelect,
  onStart,
  onArrange,
}: Props) {
  const place = { machine: machine.id, id: workspace.id };
  const current = view !== undefined && view.checkout === undefined;
  const open = useOpen(place);
  const [deleting, setDeleting] = useState(false);
  const [naming, setNaming] = useState(false);
  const main = mainCheckout(workspace);
  const dragging = useMoving();
  const over = useOverWorkspace(workspace.id);
  const index = siblings.indexOf(workspace.id);
  const moving =
    dragging?.machine === machine.id && dragging.id !== workspace.id
      ? dragging.id
      : undefined;

  const start = () => {
    onStart(place, workspace.folder);
  };

  const branch = () => {
    setNaming(true);
  };

  const remove = () => {
    setDeleting(true);
  };

  const raise = () => {
    onArrange(workspace.id, siblings[index - 1] ?? null);
  };

  const lower = () => {
    onArrange(workspace.id, siblings[index + 2] ?? null);
  };

  return (
    <SidebarMenuItem
      onDragOver={(event) => {
        if (!moving) {
          return;
        }

        const row = event.currentTarget
          .querySelector("[data-heading]")
          ?.getBoundingClientRect();
        const edge =
          row && event.clientY < row.top + row.height / 2 ? "above" : "below";
        const before = beforeOf(siblings, moving, workspace.id, edge);
        if (!changes(siblings, moving, before)) {
          hoverWorkspace(undefined);
          return;
        }

        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        hoverWorkspace({ id: workspace.id, edge });
      }}
      onDragLeave={(event) => {
        if (
          event.relatedTarget instanceof Node &&
          event.currentTarget.contains(event.relatedTarget)
        ) {
          return;
        }

        hoverWorkspace(undefined);
      }}
      onDrop={(event) => {
        if (!moving || !over) {
          return;
        }

        event.preventDefault();
        onArrange(moving, beforeOf(siblings, moving, workspace.id, over));
        hoverWorkspace(undefined);
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
      <Collapsible open={open}>
        <ContextMenu disabled={down}>
          <ContextMenuTrigger
            render={
              <SidebarMenuButton
                data-heading
                {...(down ? {} : reorderProps(place))}
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
            <span
              className={cn(
                "min-w-0 flex-1 truncate",
                workspace.missing && "text-muted-foreground",
              )}
            >
              {workspace.name}
            </span>
            {main?.branch && (
              <span className="max-w-1/3 min-w-0 shrink-0 truncate font-mono type-caption text-muted-foreground">
                {main.branch}
              </span>
            )}
            {main && <BranchStats changes={changesOf(workspace, main)} />}
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
                {main && (
                  <ContextMenuItem onClick={branch}>
                    New worktree
                  </ContextMenuItem>
                )}
                <ContextMenuSeparator />
              </>
            )}
            <ContextMenuItem disabled={index <= 0} onClick={raise}>
              Move up
            </ContextMenuItem>
            <ContextMenuItem
              disabled={index === siblings.length - 1}
              onClick={lower}
            >
              Move down
            </ContextMenuItem>
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
                  {main && (
                    <DropdownMenuItem onClick={branch}>
                      New worktree
                    </DropdownMenuItem>
                  )}
                  <DropdownMenuSeparator />
                </>
              )}
              <DropdownMenuItem disabled={index <= 0} onClick={raise}>
                Move up
              </DropdownMenuItem>
              <DropdownMenuItem
                disabled={index === siblings.length - 1}
                onClick={lower}
              >
                Move down
              </DropdownMenuItem>
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
