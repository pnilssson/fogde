import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
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

  const busy = status === "opening";

  return (
    <div className="flex items-center gap-2 p-2" role="status" aria-busy={busy}>
      {busy && <Spinner aria-hidden />}
      <span className="type-caption">{notice}</span>
      {!busy && (
        <Button size="sm" variant="secondary" onClick={onRetry}>
          Reopen terminal
        </Button>
      )}
    </div>
  );
}
