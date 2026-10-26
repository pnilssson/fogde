import { DotsThreeIcon, GitBranchIcon } from "@phosphor-icons/react";

import { Keys } from "@/components/keys";
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
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import {
  type RowsProps,
  ViewRows,
} from "@/features/workspaces/components/view-rows";
import { RemoveWorktree } from "@/features/workspaces/components/remove-worktree";
import { useRemoval } from "@/features/workspaces/hooks/use-worktrees";
import { checkoutName } from "@/features/workspaces/lib/checkouts";
import type { View } from "@/features/workspaces/lib/selection";
import type { Checkout } from "@/generated/protocol";
import { shortcuts } from "@/lib/shortcuts";

interface Props extends RowsProps {
  checkout: Checkout;
  current: View | undefined;
  down: boolean;
}

export function WorktreeItem({ checkout, current, down, ...props }: Props) {
  const { machine, workspace, onSelect, onStart } = props;
  const own = {
    machine: machine.id,
    id: workspace.id,
    checkout: checkout.path,
  };
  const selected = current?.checkout === checkout.path;
  const name = checkoutName(checkout);
  const removal = useRemoval(machine, workspace, checkout.path);

  const start = () => {
    onStart(own, checkout.path);
  };

  const remove = () => {
    removal.request();
  };

  return (
    <SidebarMenuSubItem>
      <ContextMenu disabled={down}>
        <ContextMenuTrigger
          render={
            <SidebarMenuSubButton
              size="sm"
              className="h-8"
              isActive={selected}
              render={<button type="button" />}
              onClick={() => {
                onSelect(own);
              }}
            />
          }
        >
          <GitBranchIcon />
          <span className="min-w-0 flex-1 truncate">{name}</span>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onClick={start}>
            New tile
            <ContextMenuShortcut>
              <Keys letter={shortcuts.tile} />
            </ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem variant="destructive" onClick={remove}>
            Remove
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      {!down && (
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <SidebarMenuAction
                aria-label={`Actions for ${name}`}
                className="pointer-events-none opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 aria-expanded:opacity-100"
              />
            }
          >
            <DotsThreeIcon />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onClick={start}>
              New tile
              <DropdownMenuShortcut>
                <Keys letter={shortcuts.tile} />
              </DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem variant="destructive" onClick={remove}>
              Remove
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
      <SidebarMenuSub className="border-l-0 empty:hidden">
        <ViewRows {...props} view={own} />
      </SidebarMenuSub>
      <RemoveWorktree
        machine={machine}
        workspace={workspace}
        checkout={checkout}
        open={removal.confirming}
        onOpenChange={removal.setConfirming}
      />
    </SidebarMenuSubItem>
  );
}
