import { cn } from "cn";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

type Status = "connected" | "connecting" | "disconnected";

const labels: Record<Status, string> = {
  connected: "Connected",
  connecting: "Connecting",
  disconnected: "Not connected",
};

interface Props {
  name: string;
  status: Status;
  onConnect: () => void;
}

export function Connection({ name, status, onConnect }: Props) {
  const badge = (
    <Badge variant="secondary">
      {status === "connecting" ? (
        <Spinner aria-hidden />
      ) : (
        <span
          aria-hidden
          className={cn(
            "size-2 rounded-full",
            status === "connected" ? "bg-success" : "bg-destructive",
          )}
        />
      )}
      {labels[status]}
    </Badge>
  );

  return (
    <div className="flex h-9 w-full items-center justify-between">
      {status === "connected" ? (
        <span className="px-2 type-body">{name}</span>
      ) : (
        <Button
          variant="secondary"
          aria-busy={status === "connecting"}
          onClick={() => {
            if (status === "disconnected") {
              onConnect();
            }
          }}
        >
          {name}
        </Button>
      )}
      {badge}
    </div>
  );
}
