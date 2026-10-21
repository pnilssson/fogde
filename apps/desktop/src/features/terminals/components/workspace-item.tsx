import { CaretRightIcon, DotsThreeIcon } from "@phosphor-icons/react";
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
import { DeleteWorkspace } from "@/features/terminals/components/delete-workspace";
import { StateIcon } from "@/features/terminals/components/state-icon";
import { WorkspaceTree } from "@/features/terminals/components/workspace-tree";
import { setOpen, useOpen } from "@/features/terminals/lib/layouts";
import type { Selection } from "@/features/terminals/lib/selection";
import { attention } from "@/features/terminals/lib/status";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  current: boolean;
  active: Selection | undefined;
  down: boolean;
  removable: boolean;
  onSelect: (workspace: Selection, terminal?: Selection) => void;
  onStart: (workspace: Selection, folder: string) => void;
}

export function WorkspaceItem({
  machine,
  workspace,
  current,
  active,
  down,
  removable,
  onSelect,
  onStart,
}: Props) {
  const place = { machine: machine.id, id: workspace.id };
  const empty =
    workspace.terminals.length === 0 && workspace.checkouts.length < 2;
  const open = useOpen(place) && !empty;
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
                aria-expanded={open}
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
                open && "rotate-90",
              )}
            />
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
            {removable && (
              <>
                <ContextMenuSeparator />
                <ContextMenuItem variant="destructive" onClick={remove}>
                  Delete
                </ContextMenuItem>
              </>
            )}
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
              {removable && (
                <>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem variant="destructive" onClick={remove}>
                    Delete
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        )}
        <CollapsibleContent>
          <WorkspaceTree
            machine={machine}
            workspace={workspace}
            current={current}
            active={active}
            onSelect={(selection) => {
              onSelect(place, selection);
            }}
            onStart={(folder) => {
              onStart(place, folder);
            }}
          />
        </CollapsibleContent>
      </Collapsible>
      {removable && (
        <DeleteWorkspace
          machine={machine}
          workspace={workspace}
          open={deleting}
          onOpenChange={setDeleting}
        />
      )}
    </SidebarMenuItem>
  );
}
