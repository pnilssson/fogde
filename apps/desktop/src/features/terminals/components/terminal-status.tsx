import { TileStatus } from "@/components/tile-status";
import { useConnection } from "@/hooks/use-connection";
import type { Machine } from "@/lib/machines";

export type Status = "opening" | "ready" | "ended" | "failed";

interface Props {
  machine: Machine;
  status: Status;
  onRetry: () => void;
}

const messages: Record<Status, string | undefined> = {
  ready: undefined,
  opening: "Opening terminal…",
  ended: "The terminal ended",
  failed: "Could not show the terminal",
};

export function TerminalStatus({ machine, status, onRetry }: Props) {
  const connection = useConnection(machine);
  const notice = messages[status];
  if (connection.state !== "connected" || !notice) {
    return null;
  }

  return status === "opening" ? (
    <TileStatus notice={notice} busy />
  ) : (
    <TileStatus notice={notice} retry="Reopen terminal" onRetry={onRetry} />
  );
}
