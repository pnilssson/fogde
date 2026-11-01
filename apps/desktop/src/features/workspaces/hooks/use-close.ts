import {
  useCloseTerminal,
  useClosing,
} from "@/features/workspaces/hooks/use-terminals";
import { ask } from "@/features/workspaces/lib/confirm";
import { busy } from "@/features/workspaces/lib/status";
import type { Terminal } from "@/generated/protocol";
import { useConnection } from "@/hooks/use-connection";
import type { Machine } from "@/lib/machines";

export function useCloseRequest(machine: Machine) {
  const close = useCloseTerminal(machine);

  return (terminal: Terminal) => {
    if (busy(terminal.status)) {
      ask({ kind: "close", machine, terminal });
      return;
    }

    close.mutate(terminal);
  };
}

export function useClose(machine: Machine, terminal: Terminal) {
  const request = useCloseRequest(machine);
  const connection = useConnection(machine);
  const pending = useClosing(machine, terminal);

  return {
    closable: connection.state === "connected" && !pending,
    request: () => {
      request(terminal);
    },
  };
}
