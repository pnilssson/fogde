import { useContext } from "react";

import { ChangesPanel } from "@/features/github/components/changes-panel";
import { useTiles } from "@/features/workspaces/lib/layouts";
import {
  findTerminal,
  findWorkspace,
  viewFolder,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";

export function WorkspaceChanges() {
  const { groups, workspace } = useContext(WorkspaceContext);
  const active = useTiles((tiles) => tiles.active);
  const found = findWorkspace(groups, workspace);
  if (!found || found.workspace.missing) {
    return null;
  }

  const machine = found.group.machine;
  const focused =
    active?.machine === machine.id ? findTerminal(groups, active) : undefined;
  const terminal =
    focused?.workspace.id === found.workspace.id ? focused.terminal : undefined;

  return (
    <ChangesPanel
      machine={machine}
      path={terminal?.folder ?? viewFolder(found.workspace, workspace)}
      agent={terminal?.agent ? terminal.id : null}
    />
  );
}
