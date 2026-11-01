import { NameDialog } from "@/components/name-dialog";
import { useRenameMachine } from "@/features/settings/hooks/use-rename-machine";
import { computer, type Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function RenameMachine({ machine, open, onOpenChange }: Props) {
  const rename = useRenameMachine(machine, () => {
    onOpenChange(false);
  });

  return (
    <NameDialog
      open={open}
      title={`Rename ${machine.name}`}
      action="Rename"
      busy="Renaming…"
      placeholder={machine.target ?? computer.name}
      initial={machine.name}
      optional
      onOpenChange={onOpenChange}
      onSubmit={(name) => rename.mutateAsync(name)}
    />
  );
}
