import {
  PlugsConnectedIcon,
  FolderSimplePlusIcon,
} from "@phosphor-icons/react";
import type { UseQueryResult } from "@tanstack/react-query";
import { cn } from "cn";
import { useEffect, useRef } from "react";

import { BarTitle } from "@/components/bar-title";
import { Bar, Layer } from "@/components/frame";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { ScrollArea } from "@/components/ui/scroll-area";
import { SidebarGroupAction, SidebarMenu } from "@/components/ui/sidebar";
import { Spinner } from "@/components/ui/spinner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useNewWorkspace } from "@/features/workspaces/components/new-workspace";
import { WorkspaceItem } from "@/features/workspaces/components/workspace-item";
import { useStatus } from "@/features/workspaces/hooks/use-terminals";
import { useArrangeWorkspace } from "@/features/workspaces/hooks/use-workspaces";
import { changes } from "@/features/workspaces/lib/order";
import type { Selection, View } from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const action = "static w-8 text-muted-foreground";

interface Props {
  machine: Machine;
  workspaces: UseQueryResult<Workspace[]>;
  selected: View | undefined;
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
  height: number | undefined;
  onMeasure: (machine: string, height: number) => void;
}

export function MachineGroup({
  machine,
  workspaces,
  selected,
  onSelect,
  onStart,
  height,
  onMeasure,
}: Props) {
  const bar = useRef<HTMLElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const status = useStatus(machine, workspaces);
  const down = status.disconnected || status.failed;
  const loading = workspaces.isPending && !down;
  const newWorkspace = useNewWorkspace(machine);
  const arrange = useArrangeWorkspace(machine);
  const ids = workspaces.data?.map(({ id }) => id) ?? [];

  const create = () => {
    void newWorkspace.start();
  };

  useEffect(() => {
    const top = bar.current;
    const rows = content.current;
    if (!top || !rows) {
      return;
    }

    const observer = new ResizeObserver(() => {
      onMeasure(machine.id, top.offsetHeight + rows.offsetHeight);
    });
    observer.observe(top);
    observer.observe(rows);

    return () => {
      observer.disconnect();
    };
  }, [machine.id, onMeasure]);

  return (
    <Layer
      aria-label={machine.name}
      style={height === undefined ? undefined : { height }}
      className={height === undefined ? "flex-1" : "shrink-0"}
    >
      <Bar ref={bar}>
        <ContextMenu disabled={down || loading}>
          <ContextMenuTrigger
            render={
              <div
                className={cn("flex min-w-0 flex-1 pl-2", down && "opacity-50")}
              />
            }
          >
            <BarTitle title={machine.name} muted />
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
                <PlugsConnectedIcon />
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
      </Bar>
      {newWorkspace.dialog}
      <ScrollArea className="min-h-0 flex-1">
        <div ref={content} className="px-2 pb-2">
          <SidebarMenu className={cn(down && "opacity-50")}>
            {workspaces.data?.map((workspace) => (
              <WorkspaceItem
                key={workspace.id}
                machine={machine}
                workspace={workspace}
                view={
                  selected?.machine === machine.id &&
                  selected.id === workspace.id
                    ? selected
                    : undefined
                }
                down={down}
                siblings={ids}
                onSelect={onSelect}
                onStart={onStart}
                onArrange={(workspace, before) => {
                  if (changes(ids, workspace, before)) {
                    arrange.mutate({ workspace, before });
                  }
                }}
              />
            ))}
          </SidebarMenu>
        </div>
      </ScrollArea>
    </Layer>
  );
}
