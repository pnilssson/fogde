import { useContext } from "react";

import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { DeleteWorkspace } from "@/features/terminals/components/delete-workspace";
import { StartActions } from "@/features/terminals/components/start-actions";
import { WorktreeActions } from "@/features/terminals/components/worktree-actions";
import { useConnection } from "@/features/terminals/hooks/use-terminals";
import { checkoutName, checkoutOf } from "@/features/terminals/lib/checkouts";
import {
  findWorkspace,
  WorkspaceContext,
} from "@/features/terminals/lib/selection";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  folder?: string | undefined;
}

export function StartPage({ folder }: Props) {
  const { groups, workspace } = useContext(WorkspaceContext);
  const found = findWorkspace(groups, workspace);
  const target = folder ?? found?.workspace.folder;
  const checkout =
    found && target ? checkoutOf(found.workspace, target) : undefined;
  const worktree = checkout !== undefined && !checkout.main;
  const place = worktree ? checkoutName(checkout) : found?.workspace.name;
  const home = found?.group.workspaces.data?.[0];

  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>
          {place ? `Start in ${place}` : "Start something"}
        </EmptyTitle>
        {target && <EmptyDescription>{target}</EmptyDescription>}
      </EmptyHeader>
      {found && target && (
        <Start
          machine={found.group.machine}
          workspace={found.workspace}
          folder={target}
          removable={!worktree && home?.id !== found.workspace.id}
        />
      )}
    </Empty>
  );
}

interface StartProps {
  machine: Machine;
  workspace: Workspace;
  folder: string;
  removable: boolean;
}

function Start({ machine, workspace, folder, removable }: StartProps) {
  const connection = useConnection(machine);
  if (connection.state !== "connected") {
    return null;
  }

  return (
    <EmptyContent>
      <div className="flex w-full flex-col items-center gap-8">
        <StartActions
          machine={machine}
          workspace={workspace.id}
          folder={folder}
        />
        <WorktreeActions
          machine={machine}
          workspace={workspace}
          folder={folder}
        />
        {removable && (
          <DeleteWorkspace machine={machine} workspace={workspace} />
        )}
      </div>
    </EmptyContent>
  );
}
