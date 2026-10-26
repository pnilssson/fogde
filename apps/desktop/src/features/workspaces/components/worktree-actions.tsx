import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { WorktreeForm } from "@/features/workspaces/components/new-worktree";
import { RemoveWorktree } from "@/features/workspaces/components/remove-worktree";
import { useRemoval } from "@/features/workspaces/hooks/use-worktrees";
import { checkoutOf, mainCheckout } from "@/features/workspaces/lib/checkouts";
import { closeStart } from "@/features/workspaces/lib/opening";
import type { Checkout, Workspace } from "@/generated/protocol";
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
    <Removal machine={machine} workspace={workspace} checkout={checkout} />
  );
}

interface RemovalProps {
  machine: Machine;
  workspace: Workspace;
  checkout: Checkout;
}

function Removal({ machine, workspace, checkout }: RemovalProps) {
  const removal = useRemoval(machine, workspace, checkout.path);

  return (
    <>
      <Button
        variant="secondary"
        size="sm"
        disabled={removal.pending}
        aria-busy={removal.pending}
        onClick={() => {
          removal.request();
        }}
      >
        {removal.pending && <Spinner aria-hidden data-icon="inline-start" />}
        {removal.pending ? "Removing…" : "Remove worktree"}
      </Button>
      <RemoveWorktree
        machine={machine}
        workspace={workspace}
        checkout={checkout}
        open={removal.confirming}
        onOpenChange={removal.setConfirming}
      />
    </>
  );
}
