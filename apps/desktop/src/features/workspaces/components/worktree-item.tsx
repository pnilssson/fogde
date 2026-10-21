import {
  CaretRightIcon,
  DotsThreeIcon,
  GitBranchIcon,
} from "@phosphor-icons/react";
import { cn } from "cn";

import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenuAction,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import {
  type RowsProps,
  ViewRows,
} from "@/features/workspaces/components/view-rows";
import { useShown } from "@/features/workspaces/hooks/use-shown";
import { checkoutName } from "@/features/workspaces/lib/checkouts";
import { setOpen, useOpen } from "@/features/workspaces/lib/layouts";
import { inView, type View } from "@/features/workspaces/lib/selection";
import { attention } from "@/features/workspaces/lib/status";
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
  const open = useOpen(own);
  const shown = useShown(workspace, own);
  const name = checkoutName(checkout);

  const start = () => {
    onStart(own, checkout.path);
  };

  return (
    <SidebarMenuSubItem>
      <Collapsible open={open}>
        <ContextMenu disabled={down}>
          <ContextMenuTrigger
            render={
              <SidebarMenuSubButton
                size="sm"
                isActive={selected}
                aria-expanded={open}
                render={<button type="button" />}
                onClick={() => {
                  if (selected) {
                    setOpen(own, !open);

                    return;
                  }

                  onSelect(own);
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
            <GitBranchIcon />
            <span className="min-w-0 flex-1 truncate">{name}</span>
            {!open && !down && (
              <StateIcon
                status={attention(
                  workspace.terminals.filter((terminal) =>
                    inView(own, terminal),
                  ),
                )}
                className="group-focus-within/menu-sub-item:invisible group-hover/menu-sub-item:invisible"
              />
            )}
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem onClick={start}>New tile</ContextMenuItem>
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
            </DropdownMenuContent>
          </DropdownMenu>
        )}
        {shown && (
          <CollapsibleContent>
            <SidebarMenuSub>
              <ViewRows {...props} view={own} />
            </SidebarMenuSub>
          </CollapsibleContent>
        )}
      </Collapsible>
    </SidebarMenuSubItem>
  );
}
