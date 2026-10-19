import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { useEffect } from "react";

import { SidebarInset } from "@/components/ui/sidebar";
import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { SettingsDialog } from "@/features/settings/components/settings-dialog";
import { StatusBar } from "@/features/terminals/components/status-bar";
import { WorkspaceSidebar } from "@/features/terminals/components/workspace-sidebar";
import { useGroups } from "@/features/terminals/hooks/use-terminals";
import { keepMachines, useTiles } from "@/features/terminals/lib/layouts";
import {
  findTerminal,
  resolveWorkspace,
  WorkspaceContext,
} from "@/features/terminals/lib/selection";
import {
  openStart,
  selectWorkspace,
  show,
} from "@/features/terminals/lib/tiles";
import { useMachines } from "@/hooks/use-machines";
import { useRetryOnReturn } from "@/hooks/use-retry-on-return";
import { entryId, machinesQuery } from "@/lib/machines";

export const Route = createFileRoute("/_onboarded")({
  beforeLoad: async ({ context }) => {
    const [onboarded, machines] = await Promise.all([
      context.queries.query(onboardedQuery),
      context.queries.query(machinesQuery),
    ]);
    if (onboarded && machines.length > 0) {
      return;
    }

    const [only] = machines;
    if (!onboarded && only && machines.length === 1) {
      throw redirect({
        to: "/onboarding/github",
        search: { machine: entryId(only) },
      });
    }

    throw redirect({ to: "/onboarding/where" });
  },
  component: function WorkspaceRoute() {
    const machines = useMachines();
    const groups = useGroups(machines);
    const active = useTiles((tiles) => tiles.active);
    const stored = useTiles((tiles) => tiles.workspace);
    const workspace = resolveWorkspace(groups, stored);
    const terminal = active && findTerminal(groups, active)?.terminal;

    useRetryOnReturn(machines);

    useEffect(() => {
      if (machines.length > 0) {
        keepMachines(machines.map(({ id }) => id));
      }
    }, [machines]);

    useEffect(() => {
      if (workspace) {
        selectWorkspace(workspace);
      }
    }, [workspace]);

    return (
      <WorkspaceContext value={{ groups, workspace }}>
        <div className="flex h-full min-h-0 flex-col">
          <div className="relative flex min-h-0 flex-1">
            <WorkspaceSidebar
              groups={groups}
              selected={workspace}
              active={active}
              onSelect={(next, selection) => {
                selectWorkspace(next);
                if (selection) {
                  show(selection);
                }
              }}
              onStart={(next, folder) => {
                selectWorkspace(next);
                openStart(folder);
              }}
            />
            <SidebarInset className="min-h-0 min-w-0 overflow-hidden bg-transparent">
              <Outlet />
            </SidebarInset>
          </div>
          <StatusBar start={<SettingsDialog />} folder={terminal?.folder} />
        </div>
      </WorkspaceContext>
    );
  },
});
