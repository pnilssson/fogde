import { createContext } from "react";
import { z } from "zod";

import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import { byAttention, needsYou } from "@/features/workspaces/lib/status";

export const Selection = z.object({ machine: z.string(), id: z.string() });

export type Selection = z.infer<typeof Selection>;

type Groups = ReturnType<typeof useGroups>;

interface Value {
  groups: Groups;
  workspace: Selection | undefined;
}

export const WorkspaceContext = createContext<Value>({
  groups: [],
  workspace: undefined,
});

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
  const terminal = group?.workspaces.data
    ?.flatMap(({ terminals }) => terminals)
    .find((current) => current.id === id);

  return group && terminal ? { machine: group.machine, terminal } : undefined;
}

export function resolveWorkspace(
  groups: Groups,
  stored: Selection | undefined,
): Selection | undefined {
  const group = findGroup(groups, stored?.machine);
  const known =
    !group?.workspaces.data ||
    group.workspaces.data.some(({ id }) => id === stored?.id);
  if (stored && group && known) {
    return stored;
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
          workspace: { machine: machine.id, id: workspace.id },
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
