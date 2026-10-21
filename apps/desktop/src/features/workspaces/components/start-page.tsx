import { useContext, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { DeleteWorkspace } from "@/features/workspaces/components/delete-workspace";
import { useNewWorkspace } from "@/features/workspaces/components/new-workspace";
import { StartActions } from "@/features/workspaces/components/start-actions";
import { WorktreeActions } from "@/features/workspaces/components/worktree-actions";
import { useConnection } from "@/hooks/use-connection";
import { checkoutName, checkoutOf } from "@/features/workspaces/lib/checkouts";
import {
  findWorkspace,
  viewFolder,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";
import { isComputer, type Machine } from "@/lib/machines";

interface Props {
  folder?: string | undefined;
}

export function StartPage({ folder }: Props) {
  const { groups, workspace } = useContext(WorkspaceContext);
  const found = findWorkspace(groups, workspace);
  if (!found) {
    if (workspace) {
      return null;
    }

    const loaded = groups.find(({ workspaces }) => workspaces.data);

    return loaded ? <NoWorkspace machine={loaded.machine} /> : null;
  }

  const machine = found.group.machine;
  if (found.workspace.missing) {
    return <Missing machine={machine} workspace={found.workspace} />;
  }

  const target = folder ?? viewFolder(found.workspace, workspace);
  const checkout = checkoutOf(found.workspace, target);
  const worktree = checkout !== undefined && !checkout.main;
  const place = worktree ? checkoutName(checkout) : found.workspace.name;

  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>
          Continue working in {place} on {spoken(machine)}
        </EmptyTitle>
        <EmptyDescription>{target}</EmptyDescription>
      </EmptyHeader>
      <Start machine={machine} workspace={found.workspace} folder={target} />
    </Empty>
  );
}

function NoWorkspace({ machine }: { machine: Machine }) {
  const newWorkspace = useNewWorkspace(machine);

  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>Create a workspace</EmptyTitle>
        <EmptyDescription>
          A workspace is a folder you work in, usually a repo, with its
          terminals and agents beside it.
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button
          size="sm"
          onClick={() => {
            void newWorkspace.start();
          }}
        >
          New workspace
        </Button>
      </EmptyContent>
      {newWorkspace.dialog}
    </Empty>
  );
}

interface MissingProps {
  machine: Machine;
  workspace: Workspace;
}

function Missing({ machine, workspace }: MissingProps) {
  const [deleting, setDeleting] = useState(false);

  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>Folder not found</EmptyTitle>
        <EmptyDescription>
          <span className="wrap-anywhere">{workspace.folder}</span> was moved or
          deleted.
        </EmptyDescription>
      </EmptyHeader>
      <EmptyContent>
        <Button
          variant="destructive"
          size="sm"
          onClick={() => {
            setDeleting(true);
          }}
        >
          Delete workspace
        </Button>
      </EmptyContent>
      <DeleteWorkspace
        machine={machine}
        workspace={workspace}
        open={deleting}
        onOpenChange={setDeleting}
      />
    </Empty>
  );
}

interface StartProps {
  machine: Machine;
  workspace: Workspace;
  folder: string;
}

function Start({ machine, workspace, folder }: StartProps) {
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
      </div>
    </EmptyContent>
  );
}

function spoken(machine: Machine) {
  return isComputer(machine) ? "this computer" : machine.name;
}
