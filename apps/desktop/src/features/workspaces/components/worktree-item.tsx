import { DotsThreeIcon, GitBranchIcon } from "@phosphor-icons/react";

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
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import {
  type RowsProps,
  ViewRows,
} from "@/features/workspaces/components/view-rows";
import { useRemoveWorktree } from "@/features/workspaces/hooks/use-worktrees";
import { checkoutName } from "@/features/workspaces/lib/checkouts";
import type { View } from "@/features/workspaces/lib/selection";
import type { Checkout } from "@/generated/protocol";

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
  const removal = useRemoveWorktree(machine, workspace);

  const start = () => {
    onStart(own, checkout.path);
  };

  const remove = () => {
    removal.mutate(checkout.path);
  };

  return (
    <>
      <SidebarMenuSubItem>
        <ContextMenu disabled={down}>
          <ContextMenuTrigger
            render={
              <SidebarMenuSubButton
                size="sm"
                isActive={selected}
                render={<button type="button" />}
                className="text-muted-foreground [&>svg]:text-muted-foreground"
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
            <ContextMenuItem onClick={start}>New tile</ContextMenuItem>
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
                  className="top-1 h-5 opacity-0 group-focus-within/menu-sub-item:opacity-100 group-hover/menu-sub-item:opacity-100 aria-expanded:opacity-100"
                />
              }
            >
              <DotsThreeIcon />
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem onClick={start}>New tile</DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem variant="destructive" onClick={remove}>
                Remove
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </SidebarMenuSubItem>
      <ViewRows {...props} view={own} />
    </>
  );
}
