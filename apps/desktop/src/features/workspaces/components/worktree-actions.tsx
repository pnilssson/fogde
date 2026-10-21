import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { WorktreeForm } from "@/features/workspaces/components/new-worktree";
import { useRemoveWorktree } from "@/features/workspaces/hooks/use-worktrees";
import { checkoutOf, mainCheckout } from "@/features/workspaces/lib/checkouts";
import { closeStart } from "@/features/workspaces/lib/tiles";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  folder: string;
}

export function WorktreeActions({ machine, workspace, folder }: Props) {
  const main = mainCheckout(workspace);
  const checkout = checkoutOf(workspace, folder);
  if (!main || !checkout) {
    return null;
  }

  if (checkout.main) {
    return (
      <WorktreeForm
        machine={machine}
        workspace={workspace.id}
        repo={main.path}
        label="New worktree"
        className="w-full max-w-xs"
        onCreated={closeStart}
      />
    );
  }

  return (
    <RemoveWorktree
      machine={machine}
      workspace={workspace}
      path={checkout.path}
    />
  );
}

interface RemoveProps {
  machine: Machine;
  workspace: Workspace;
  path: string;
}

function RemoveWorktree({ machine, workspace, path }: RemoveProps) {
  const remove = useRemoveWorktree(machine, workspace);

  return (
    <Button
      variant="secondary"
      size="sm"
      disabled={remove.isPending}
      aria-busy={remove.isPending}
      onClick={() => {
        remove.mutate(path);
      }}
    >
      {remove.isPending && <Spinner aria-hidden data-icon="inline-start" />}
      {remove.isPending ? "Removing…" : "Remove worktree"}
    </Button>
  );
}
