import { GitBranchIcon } from "@phosphor-icons/react";
import { memo, useMemo } from "react";

import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import {
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { BranchStats } from "@/features/workspaces/components/branch-stats";
import {
  RowActions,
  RowContext,
} from "@/features/workspaces/components/row-menu";
import {
  type RowsProps,
  ViewRows,
} from "@/features/workspaces/components/view-rows";
import { useRemoval } from "@/features/workspaces/hooks/use-worktrees";
import { changesOf, checkoutName } from "@/features/workspaces/lib/checkouts";
import type { View } from "@/features/workspaces/lib/selection";
import type { Checkout } from "@/generated/protocol";
import { shortcuts } from "@/lib/shortcuts";

interface Props extends RowsProps {
  checkout: Checkout;
  current: View | undefined;
  down: boolean;
}

export const WorktreeItem = memo(function WorktreeItem({
  checkout,
  current,
  down,
  ...props
}: Props) {
  const { machine, workspace, onSelect, onStart } = props;
  const own = useMemo(
    () => ({ machine: machine.id, id: workspace.id, checkout: checkout.path }),
    [machine.id, workspace.id, checkout.path],
  );
  const selected = current?.checkout === checkout.path;
  const name = checkoutName(checkout);
  const removal = useRemoval(machine, workspace, checkout);
  const groups = [
    [
      {
        label: "New tile",
        shortcut: shortcuts.tile,
        onSelect: () => {
          onStart(own, checkout.path);
        },
      },
    ],
    [{ label: "Remove", destructive: true, onSelect: removal.request }],
  ];

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
          <BranchStats changes={changesOf(workspace, checkout)} />
        </ContextMenuTrigger>
        <RowContext groups={groups} />
      </ContextMenu>
      {!down && <RowActions name={name} groups={groups} />}
      <SidebarMenuSub className="border-l-0 empty:hidden">
        <ViewRows {...props} view={own} />
      </SidebarMenuSub>
    </SidebarMenuSubItem>
  );
});
