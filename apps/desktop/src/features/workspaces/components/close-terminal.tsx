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
import { useCloseTerminal } from "@/features/workspaces/hooks/use-terminals";
import type { Terminal } from "@/generated/protocol";
import { agentName } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  terminal: Terminal;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function warning({ agent, status }: Terminal) {
  const who = agent ? agentName[agent] : "The terminal";

  return status === "blocked"
    ? `${who} is waiting for your input. Closing stops it.`
    : `${who} is still working. Closing stops it.`;
}

export function CloseTerminal({
  machine,
  terminal,
  open,
  onOpenChange,
}: Props) {
  const close = useCloseTerminal(machine);

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{`Close ${terminal.name}?`}</AlertDialogTitle>
          <AlertDialogDescription>{warning(terminal)}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            size="sm"
            onClick={() => {
              close.mutate(terminal);
              onOpenChange(false);
            }}
          >
            Close
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
