import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

type Status = "connected" | "connecting" | "disconnected";

interface Props {
  name: string;
  status: Status;
  onConnect: () => void;
}

export function Connection({ name, status, onConnect }: Props) {
  return (
    <div className="flex h-9 w-full items-center justify-between">
      <span className="type-body">{name}</span>
      {status === "connected" ? (
        <Badge variant="secondary">
          <span aria-hidden className="size-2 rounded-full bg-success" />
          Connected
        </Badge>
      ) : (
        <Button
          variant="secondary"
          size="sm"
          aria-label={`Connect ${name}`}
          aria-busy={status === "connecting"}
          onClick={() => {
            if (status === "disconnected") {
              onConnect();
            }
          }}
        >
          {status === "connecting" && (
            <Spinner aria-hidden data-icon="inline-start" />
          )}
          Connect
        </Button>
      )}
    </div>
  );
}
