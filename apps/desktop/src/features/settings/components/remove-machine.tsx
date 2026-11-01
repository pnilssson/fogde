import { ConfirmDialog } from "@/components/confirm-dialog";
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
    <ConfirmDialog
      open={open}
      onOpenChange={onOpenChange}
      title={`Remove ${machine.name}?`}
      description={`Its workspaces, terminals and agents stay ${place}. Add it again to pick up where you left off.`}
      action="Remove"
      busy="Removing…"
      pending={remove.isPending}
      onConfirm={() => {
        remove.mutate(machine);
      }}
    />
  );
}
