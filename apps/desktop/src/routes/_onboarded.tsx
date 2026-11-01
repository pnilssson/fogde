import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { cn } from "cn";
import { useEffect, useState } from "react";

import { Pages } from "@/app/pages";
import { PageNav } from "@/app/page-nav";
import { SidebarFoot } from "@/app/sidebar-foot";

import { SidebarToggle } from "@/components/sidebar-toggle";
import { TitleBar } from "@/components/title-bar";
import { Shell } from "@/components/shell";
import { useAgents } from "@/features/agents/hooks/use-agents";
import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { PaletteNotes } from "@/features/notes/components/palette-notes";
import { useNotesChanges } from "@/features/notes/hooks/use-notes-changes";
import { CommandPalette } from "@/features/workspaces/components/command-palette";
import { WorkspaceChanges } from "@/app/changes";
import { PanelToggle } from "@/features/github/components/panel-toggle";
import { usePanel } from "@/features/github/lib/panel";
import { SearchButton } from "@/features/workspaces/components/search-button";
import { WorkspaceSidebar } from "@/features/workspaces/components/workspace-sidebar";
import { useAttention } from "@/features/workspaces/hooks/use-attention";
import { useBadge } from "@/features/workspaces/hooks/use-badge";
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
import { SideSlot } from "@/lib/side-slot";
import { useRetryOnReturn } from "@/hooks/use-retry-on-return";
import { entryId, machinesQuery } from "@/lib/machines";
import { closePage, leavePage, usePage } from "@/lib/page";

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
    const sided = page === "github" || page === "notes" || page === "settings";
    const [slot, setSlot] = useState<HTMLElement | null>(null);
    const panel = usePanel((state) => state.open);

    useRetryOnReturn(machines);
    useAttention(machines);
    useBadge(groups);
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
            start={<SidebarToggle />}
            center={<SearchButton />}
            end={!page && workspace && <PanelToggle />}
          />
          <div className="relative flex min-h-0 flex-1 flex-col">
            <Shell
              side={
                <>
                  <PageNav machine={workspace?.machine} />
                  {sided ? (
                    <div
                      ref={setSlot}
                      className="flex min-h-0 flex-1 flex-col"
                    />
                  ) : (
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
                    />
                  )}
                  <SidebarFoot machines={machines} />
                </>
              }
            >
              <div
                className={cn(
                  "flex min-h-0 flex-1 flex-col",
                  page && "opacity-0",
                )}
                inert={page !== undefined}
              >
                <div className="flex min-h-0 flex-1 gap-2">
                  <div className="flex min-h-0 min-w-0 flex-1 flex-col">
                    <Outlet />
                  </div>
                  {panel && <WorkspaceChanges />}
                </div>
              </div>
              {page && (
                <div className="absolute inset-0 flex flex-col px-2 pb-2">
                  <SideSlot value={slot}>
                    <Pages groups={groups} workspace={workspace} />
                  </SideSlot>
                </div>
              )}
            </Shell>
          </div>
          <CommandPalette notes={(props) => <PaletteNotes {...props} />} />
        </div>
      </WorkspaceContext>
    );
  },
});
