import {
  ArrowClockwiseIcon,
  DesktopIcon,
  FolderSimplePlusIcon,
  HardDrivesIcon,
} from "@phosphor-icons/react";
import type { UseQueryResult } from "@tanstack/react-query";
import { cn } from "cn";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
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
import { useNewWorkspace } from "@/features/workspaces/components/new-workspace";
import { WorkspaceItem } from "@/features/workspaces/components/workspace-item";
import { useStatus } from "@/features/workspaces/hooks/use-terminals";
import type { Selection } from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";
import { isComputer, type Machine } from "@/lib/machines";

const action = "static w-7 text-muted-foreground [&>svg]:size-5";

interface Props {
  machine: Machine;
  workspaces: UseQueryResult<Workspace[]>;
  selected: Selection | undefined;
  onSelect: (workspace: Selection, terminal?: Selection) => void;
  onStart: (workspace: Selection, folder: string) => void;
}

export function MachineGroup({
  machine,
  workspaces,
  selected,
  onSelect,
  onStart,
}: Props) {
  const status = useStatus(machine, workspaces);
  const down = status.disconnected || status.failed;
  const loading = workspaces.isPending && !down;
  const newWorkspace = useNewWorkspace(machine);

  const create = () => {
    void newWorkspace.start();
  };

  return (
    <SidebarGroup className="py-0">
      <div className="flex items-center">
        <ContextMenu disabled={down || loading}>
          <ContextMenuTrigger
            render={
              <SidebarGroupLabel
                className={cn(
                  "h-10 min-w-0 flex-1 gap-2 type-title text-muted-foreground [&>svg]:size-5",
                  down && "opacity-50",
                )}
              />
            }
          >
            {isComputer(machine) ? <DesktopIcon /> : <HardDrivesIcon />}
            <span className="truncate">{machine.name}</span>
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
                  className={action}
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
              {status.retrying ? (
                <Spinner aria-hidden />
              ) : (
                <ArrowClockwiseIcon />
              )}
            </TooltipTrigger>
            <TooltipContent side="right">
              {status.error ?? (status.disconnected ? "Reconnect" : "Retry")}
            </TooltipContent>
          </Tooltip>
        ) : loading ? (
          <SidebarGroupAction
            render={<span />}
            className={cn(action, "pointer-events-none")}
          >
            <Spinner aria-label={`Connecting to ${machine.name}`} />
          </SidebarGroupAction>
        ) : (
          <Tooltip>
            <TooltipTrigger
              render={
                <SidebarGroupAction
                  className={action}
                  aria-label={`New workspace on ${machine.name}`}
                  onClick={create}
                />
              }
            >
              <FolderSimplePlusIcon />
            </TooltipTrigger>
            <TooltipContent side="right">New workspace</TooltipContent>
          </Tooltip>
        )}
      </div>
      {newWorkspace.dialog}
      <SidebarMenu className={cn("pl-4", down && "opacity-50")}>
        {workspaces.data?.map((workspace) => (
          <WorkspaceItem
            key={workspace.id}
            machine={machine}
            workspace={workspace}
            current={
              selected?.machine === machine.id && selected.id === workspace.id
            }
            down={down}
            onSelect={onSelect}
            onStart={onStart}
          />
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}
