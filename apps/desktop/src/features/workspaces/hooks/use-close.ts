import { useState } from "react";

import {
  useCloseTerminal,
  useClosing,
} from "@/features/workspaces/hooks/use-terminals";
import { busy } from "@/features/workspaces/lib/status";
import type { Terminal } from "@/generated/protocol";
import { useConnection } from "@/hooks/use-connection";
import type { Machine } from "@/lib/machines";

export function useClose(machine: Machine, terminal: Terminal) {
  const close = useCloseTerminal(machine);
  const connection = useConnection(machine);
  const pending = useClosing(machine, terminal);
  const [confirming, setConfirming] = useState(false);

  const request = () => {
    if (busy(terminal.status)) {
      setConfirming(true);
      return;
    }

    close.mutate(terminal);
  };

  return {
    closable: connection.state === "connected" && !pending,
    confirming,
    setConfirming,
    request,
  };
}
