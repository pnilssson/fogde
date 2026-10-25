import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { useEffect } from "react";

import { StatusBar } from "@/components/status-bar";
import { SidebarInset } from "@/components/ui/sidebar";
import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { SettingsDialog } from "@/features/settings/components/settings-dialog";
import { UsageMeters } from "@/features/usage/components/usage-meters";
import { PaletteNotes } from "@/features/notes/components/palette-notes";
import { useNotesChanges } from "@/features/notes/hooks/use-notes-changes";
import { CommandPalette } from "@/features/workspaces/components/command-palette";
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

    useRetryOnReturn(machines);
    useAttention(machines);
    useFailures(machines);
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
          <div className="relative flex min-h-0 flex-1">
            <WorkspaceSidebar
              groups={groups}
              selected={workspace}
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
            <SidebarInset className="min-h-0 min-w-0 overflow-hidden bg-surface-3">
              <Outlet />
            </SidebarInset>
          </div>
          <StatusBar
            start={<SettingsDialog />}
            end={<UsageMeters machines={machines} />}
          />
          <CommandPalette notes={(props) => <PaletteNotes {...props} />} />
        </div>
      </WorkspaceContext>
    );
  },
});
