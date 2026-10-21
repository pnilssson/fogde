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
import { Spinner } from "@/components/ui/spinner";
import { useChangeMachine } from "@/features/settings/hooks/use-change-machine";
import { isComputer, type Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function RemoveMachine({ machine, open, onOpenChange }: Props) {
  const remove = useChangeMachine("remove", () => {
    onOpenChange(false);
  });
  const place = isComputer(machine) ? "on this computer" : "on the server";

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{`Remove ${machine.name}?`}</AlertDialogTitle>
          <AlertDialogDescription>
            {`Its workspaces, terminals and agents stay ${place}. Add it again to pick up where you left off.`}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            size="sm"
            disabled={remove.isPending}
            aria-busy={remove.isPending}
            onClick={() => {
              remove.mutate(machine);
            }}
          >
            {remove.isPending && (
              <Spinner aria-hidden data-icon="inline-start" />
            )}
            {remove.isPending ? "Removing…" : "Remove"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
