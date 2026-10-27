import { createContext } from "react";
import { z } from "zod";

import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { byAttention, needsYou } from "@/features/workspaces/lib/status";
import type { Terminal, Workspace } from "@/generated/protocol";

export const Selection = z.object({ machine: z.string(), id: z.string() });

export type Selection = z.infer<typeof Selection>;

export const View = Selection.extend({ checkout: z.string().optional() });

export type View = z.infer<typeof View>;

type Groups = ReturnType<typeof useGroups>;

interface Value {
  groups: Groups;
  workspace: View | undefined;
}

export const WorkspaceContext = createContext<Value>({
  groups: [],
  workspace: undefined,
});

export function viewOf(
  machine: string,
  workspace: Workspace,
  terminal?: Terminal,
): View {
  const view = { machine, id: workspace.id };

  return terminal?.checkout ? { ...view, checkout: terminal.checkout } : view;
}

export function sameView(a: View | undefined, b: View | undefined) {
  return (
    a?.machine === b?.machine && a?.id === b?.id && a?.checkout === b?.checkout
  );
}

export function inView(view: View, tile: { checkout: string | null }) {
  return (tile.checkout ?? undefined) === view.checkout;
}

export function viewFolder(workspace: Workspace, view: View | undefined) {
  return view?.checkout ?? workspace.folder;
}

export function worktrees(workspace: Workspace) {
  return workspace.checkouts.filter(({ main }) => !main);
}

function findGroup(groups: Groups, machine: string | undefined) {
  return groups.find((group) => group.machine.id === machine);
}

export function findWorkspace(
  groups: Groups,
  selection: Selection | undefined,
) {
  const group = findGroup(groups, selection?.machine);
  const workspace = group?.workspaces.data?.find(
    ({ id }) => id === selection?.id,
  );

  return group && workspace ? { group, workspace } : undefined;
}

export function findTerminal(groups: Groups, { machine, id }: Selection) {
  const group = findGroup(groups, machine);
  const owns = (workspace: Workspace) =>
    workspace.terminals.some((current) => current.id === id);
  const workspace = group?.workspaces.data?.find(owns);
  const terminal = workspace
    ? workspace.terminals.find((current) => current.id === id)
    : undefined;

  return group && workspace && terminal
    ? { machine: group.machine, workspace, terminal }
    : undefined;
}

export function resolveWorkspace(
  groups: Groups,
  stored: View | undefined,
): View | undefined {
  const group = findGroup(groups, stored?.machine);
  if (stored && group && !group.workspaces.data) {
    return stored;
  }

  const workspace = group?.workspaces.data?.find(({ id }) => id === stored?.id);
  if (stored && workspace) {
    const known = worktrees(workspace).some(
      ({ path }) => path === stored.checkout,
    );

    return stored.checkout && !known
      ? { machine: stored.machine, id: stored.id }
      : stored;
  }

  for (const { machine, workspaces } of groups) {
    const first = workspaces.data?.[0];
    if (first) {
      return { machine: machine.id, id: first.id };
    }
  }

  return undefined;
}

export function waiting(groups: Groups) {
  return groups.flatMap(({ machine, workspaces }) =>
    (workspaces.data ?? []).flatMap((workspace) =>
      byAttention(workspace.terminals)
        .filter(({ status }) => needsYou(status))
        .map((terminal) => ({
          workspace: viewOf(machine.id, workspace, terminal),
          terminal: { machine: machine.id, id: terminal.id },
          name: terminal.name,
          agent: terminal.agent,
          place: workspace.name,
        })),
    ),
  );
}

export function nextWaiting(groups: Groups, active: Selection | undefined) {
  const list = waiting(groups);
  const at = list.findIndex(
    ({ terminal }) =>
      terminal.machine === active?.machine && terminal.id === active.id,
  );

  return list[(at + 1) % list.length];
}
