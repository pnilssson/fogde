import { memo, useState } from "react";

import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { DeleteWorkspace } from "@/features/workspaces/components/delete-workspace";
import { DropLine } from "@/features/workspaces/components/drop-line";
import { NewWorktree } from "@/features/workspaces/components/new-worktree";
import {
  type RowAction,
  RowActions,
  RowContext,
} from "@/features/workspaces/components/row-menu";
import { WorkspaceLabel } from "@/features/workspaces/components/workspace-label";
import { WorkspaceTree } from "@/features/workspaces/components/workspace-tree";
import { useReorder } from "@/features/workspaces/hooks/use-reorder";
import { mainCheckout } from "@/features/workspaces/lib/checkouts";
import { setOpen, useOpen } from "@/features/workspaces/lib/layouts";
import { moves } from "@/features/workspaces/lib/order";
import { reorderProps, useMoving } from "@/features/workspaces/lib/reorder";
import type { Selection, View } from "@/features/workspaces/lib/selection";
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

export const WorkspaceItem = memo(function WorkspaceItem({
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
  const moving =
    dragging?.machine === machine.id && dragging.id !== workspace.id
      ? dragging.id
      : undefined;
  const drop = useReorder(workspace.id, siblings, moving, onArrange);

  const create: RowAction[] = workspace.missing
    ? []
    : [
        {
          label: "New tile",
          shortcut: shortcuts.tile,
          onSelect: () => {
            onStart(place, workspace.folder);
          },
        },
        ...(main
          ? [
              {
                label: "New worktree",
                onSelect: () => {
                  setNaming(true);
                },
              },
            ]
          : []),
      ];
  const groups = [
    create,
    moves(workspace.id, siblings, onArrange),
    [
      {
        label: "Delete",
        destructive: true,
        onSelect: () => {
          setDeleting(true);
        },
      },
    ],
  ];

  return (
    <SidebarMenuItem {...drop.target}>
      <DropLine edge={drop.over} />
      <Collapsible open={open}>
        <ContextMenu disabled={down}>
          <ContextMenuTrigger
            render={
              <SidebarMenuButton
                data-heading
                size={main?.branch ? "lg" : "default"}
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
            <WorkspaceLabel
              workspace={workspace}
              main={main}
              status={!open && !down}
            />
          </ContextMenuTrigger>
          <RowContext groups={groups} />
        </ContextMenu>
        {!down && <RowActions name={workspace.name} groups={groups} />}
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
});
