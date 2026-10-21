import { ArrowClockwiseIcon, DotsThreeIcon } from "@phosphor-icons/react";
import type { UseQueryResult } from "@tanstack/react-query";
import { cn } from "cn";
import { useState } from "react";

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
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupLabel,
  SidebarMenu,
} from "@/components/ui/sidebar";
import { Spinner } from "@/components/ui/spinner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { NewWorkspace } from "@/features/workspaces/components/new-workspace";
import { WorkspaceItem } from "@/features/workspaces/components/workspace-item";
import { useStatus } from "@/features/workspaces/hooks/use-terminals";
import type { Selection } from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspaces: UseQueryResult<Workspace[]>;
  selected: Selection | undefined;
  active: Selection | undefined;
  onSelect: (workspace: Selection, terminal?: Selection) => void;
  onStart: (workspace: Selection, folder: string) => void;
}

export function MachineGroup({
  machine,
  workspaces,
  selected,
  active,
  onSelect,
  onStart,
}: Props) {
  const status = useStatus(machine, workspaces);
  const down = status.disconnected || status.failed;
  const loading = workspaces.isPending && !down;
  const [creating, setCreating] = useState(false);

  const create = () => {
    setCreating(true);
  };

  return (
    <SidebarGroup>
      <ContextMenu disabled={down || loading}>
        <ContextMenuTrigger
          render={
            <SidebarGroupLabel
              className={cn("peer/machine", down && "opacity-50")}
            />
          }
        >
          {machine.name}
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem onClick={create}>New workspace</ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      {down ? (
        <Tooltip>
          <TooltipTrigger
            render={
              <SidebarGroupAction
                aria-label={
                  status.disconnected
                    ? `Reconnect to ${machine.name}`
                    : `Retry loading ${machine.name}`
                }
                disabled={status.retrying}
                aria-busy={status.retrying}
                onClick={status.recover}
              />
            }
          >
            {status.retrying ? <Spinner aria-hidden /> : <ArrowClockwiseIcon />}
          </TooltipTrigger>
          <TooltipContent side="right">
            {status.error ?? (status.disconnected ? "Reconnect" : "Retry")}
          </TooltipContent>
        </Tooltip>
      ) : loading ? (
        <SidebarGroupAction render={<span />} className="pointer-events-none">
          <Spinner aria-label={`Connecting to ${machine.name}`} />
        </SidebarGroupAction>
      ) : (
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <SidebarGroupAction
                aria-label={`Actions for ${machine.name}`}
                className="opacity-0 peer-hover/machine:opacity-100 hover:opacity-100 focus-visible:opacity-100 aria-expanded:opacity-100"
              />
            }
          >
            <DotsThreeIcon />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuItem onClick={create}>New workspace</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
      <NewWorkspace
        machine={machine}
        open={creating}
        onOpenChange={setCreating}
      />
      <SidebarMenu className={cn(down && "opacity-50")}>
        {workspaces.data?.map((workspace, index) => (
          <WorkspaceItem
            key={workspace.id}
            machine={machine}
            workspace={workspace}
            current={
              selected?.machine === machine.id && selected.id === workspace.id
            }
            active={active}
            down={down}
            removable={index > 0}
            onSelect={onSelect}
            onStart={onStart}
          />
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}
