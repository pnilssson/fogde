import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useRemoveWorktree } from "@/features/workspaces/hooks/use-worktrees";
import { checkoutName, working } from "@/features/workspaces/lib/checkouts";
import { activity } from "@/features/workspaces/lib/status";
import type { TerminalTile } from "@/features/workspaces/lib/tile-kinds";
import type { Checkout, Workspace } from "@/generated/protocol";
import { agents } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  checkout: Checkout;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function warning(terminals: TerminalTile[]) {
  const outcome = "Removing it closes its terminals.";
  if (terminals.length > 1) {
    return `${String(terminals.length)} terminals are still busy. ${outcome}`;
  }

  const [terminal] = terminals;
  if (!terminal) {
    return outcome;
  }

  const who = terminal.agent ? agents[terminal.agent].name : "Its terminal";

  return `${who} ${activity(terminal.status)}. ${outcome}`;
}

export function RemoveWorktree({
  machine,
  workspace,
  checkout,
  open,
  onOpenChange,
}: Props) {
  const remove = useRemoveWorktree(machine, workspace);

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{`Remove ${checkoutName(checkout)}?`}</AlertDialogTitle>
          <AlertDialogDescription>
            {warning(working(workspace, checkout.path))}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            size="sm"
            onClick={() => {
              remove.mutate(checkout.path);
              onOpenChange(false);
            }}
          >
            Remove
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
