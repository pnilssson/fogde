import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { cn } from "cn";
import { useEffect } from "react";

import { Pages } from "@/app/pages";

import { SidebarToggle } from "@/components/sidebar-toggle";
import { TitleBar } from "@/components/title-bar";
import { Shell } from "@/components/shell";
import { useAgents } from "@/features/agents/hooks/use-agents";
import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { MonitorMeters } from "@/features/monitor/components/monitor-meters";
import { UsageMeters } from "@/features/usage/components/usage-meters";
import { PaletteNotes } from "@/features/notes/components/palette-notes";
import { useNotesChanges } from "@/features/notes/hooks/use-notes-changes";
import { CommandPalette } from "@/features/workspaces/components/command-palette";
import { PageTabs } from "@/features/workspaces/components/page-tabs";
import { SearchButton } from "@/features/workspaces/components/search-button";
import { WorkspaceSidebar } from "@/features/workspaces/components/workspace-sidebar";
import { useAttention } from "@/features/workspaces/hooks/use-attention";
import { useCheckouts } from "@/features/workspaces/hooks/use-checkouts";
import { useFailures } from "@/features/workspaces/hooks/use-failures";
import { useShortcuts } from "@/features/workspaces/hooks/use-shortcuts";
import { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { useViewing } from "@/features/workspaces/hooks/use-viewing";
import { keepMachines, useTiles } from "@/features/workspaces/lib/layouts";
import {
  resolveWorkspace,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import { openStart } from "@/features/workspaces/lib/opening";
import { selectWorkspace, show } from "@/features/workspaces/lib/tiles";
import { useMachines } from "@/hooks/use-machines";
import { useRetryOnReturn } from "@/hooks/use-retry-on-return";
import { entryId, machinesQuery } from "@/lib/machines";
import { closePage, isFull, leavePage, openPage, usePage } from "@/lib/page";

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
    const { all, connected: machines } = useMachines();
    const groups = useGroups(machines);
    const stored = useTiles((tiles) => tiles.workspace);
    const workspace = resolveWorkspace(groups, stored);
    const page = usePage();
    const full = isFull(page);

    useRetryOnReturn(machines);
    useAttention(machines);
    useFailures(machines);
    useAgents(machines);
    useCheckouts(machines);
    useNotesChanges(machines);
    useViewing(machines);
    useShortcuts(groups, workspace);

    useEffect(() => {
      if (all.length > 0) {
        keepMachines(all.map(({ id }) => id));
      }
    }, [all]);

    useEffect(() => {
      if (workspace) {
        selectWorkspace(workspace);
      }
    }, [workspace]);

    return (
      <WorkspaceContext value={{ groups, workspace }}>
        <div className="flex h-full min-h-0 flex-col">
          <TitleBar
            start={
              <>
                <SidebarToggle />
                <PageTabs machine={workspace?.machine} />
              </>
            }
            center={<SearchButton />}
          />
          <div className="relative flex min-h-0 flex-1 flex-col">
            <Shell
              away={full}
              side={
                <WorkspaceSidebar
                  groups={groups}
                  selected={workspace}
                  onSelect={(next, selection) => {
                    selectWorkspace(next);
                    leavePage(() => {
                      if (selection) {
                        show(selection);
                      }
                    });
                  }}
                  onStart={(next, folder) => {
                    closePage();
                    selectWorkspace(next);
                    openStart(folder);
                  }}
                  meters={
                    <>
                      <UsageMeters machines={machines} />
                      <MonitorMeters
                        machines={machines}
                        onOpen={(machine) => {
                          openPage("monitor", machine.id);
                        }}
                      />
                    </>
                  }
                />
              }
            >
              <div
                className={cn(
                  "flex min-h-0 flex-1 flex-col",
                  page && "opacity-0",
                )}
                inert={page !== undefined}
              >
                <Outlet />
              </div>
              {page === "monitor" && (
                <div className="absolute inset-0 px-2 pb-2">
                  <Pages groups={groups} workspace={workspace} />
                </div>
              )}
            </Shell>
            {full && (
              <div className="absolute inset-0 z-20 flex flex-col">
                <Pages groups={groups} workspace={workspace} />
              </div>
            )}
          </div>
          <CommandPalette notes={(props) => <PaletteNotes {...props} />} />
        </div>
      </WorkspaceContext>
    );
  },
});
