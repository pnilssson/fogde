import {
  ArrowClockwiseIcon,
  PlusIcon,
  SquaresFourIcon,
} from "@phosphor-icons/react";
import type { UseQueryResult } from "@tanstack/react-query";
import { cn } from "cn";

import {
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSkeleton,
} from "@/components/ui/sidebar";
import { Spinner } from "@/components/ui/spinner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { NewWorkspace } from "@/features/terminals/components/new-workspace";
import { WorkspaceTree } from "@/features/terminals/components/workspace-tree";
import { useStatus } from "@/features/terminals/hooks/use-terminals";
import type { Selection } from "@/features/terminals/lib/selection";
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

  return (
    <SidebarGroup>
      <SidebarGroupLabel className={cn(down && "opacity-50")}>
        {machine.name}
      </SidebarGroupLabel>
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
      ) : (
        <NewWorkspace machine={machine} />
      )}
      <SidebarMenu className={cn(down && "opacity-50")}>
        {workspaces.isPending && !down
          ? [0, 1].map((row) => (
              <SidebarMenuItem key={row}>
                <SidebarMenuSkeleton showIcon />
              </SidebarMenuItem>
            ))
          : workspaces.data?.map((workspace) => {
              const place = { machine: machine.id, id: workspace.id };
              const current =
                selected?.machine === machine.id &&
                selected.id === workspace.id;

              return (
                <SidebarMenuItem key={workspace.id}>
                  <SidebarMenuButton
                    isActive={current}
                    onClick={() => {
                      onSelect(place);
                    }}
                  >
                    <SquaresFourIcon weight={current ? "fill" : "regular"} />
                    <span>{workspace.name}</span>
                  </SidebarMenuButton>
                  {!down && (
                    <Tooltip>
                      <TooltipTrigger
                        render={
                          <SidebarMenuAction
                            showOnHover
                            aria-label={`New tile in ${workspace.name}`}
                            onClick={() => {
                              onStart(place, workspace.folder);
                            }}
                          />
                        }
                      >
                        <PlusIcon />
                      </TooltipTrigger>
                      <TooltipContent side="right">New tile</TooltipContent>
                    </Tooltip>
                  )}
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
                </SidebarMenuItem>
              );
            })}
      </SidebarMenu>
    </SidebarGroup>
  );
}
