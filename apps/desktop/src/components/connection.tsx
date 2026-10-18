import type { ReactNode } from "react";

import { DoneBadge } from "@/components/done-badge";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

type Status = "connected" | "connecting" | "disconnected";

interface Props {
  name: string;
  icon?: ReactNode;
  action?: string;
  pending?: string;
  status: Status;
  onConnect: () => void;
}

export function Connection({
  name,
  icon,
  action = "Connect",
  pending = action,
  status,
  onConnect,
}: Props) {
  return (
    <div className="flex h-9 w-full items-center justify-between">
      <span className="flex items-center gap-2 type-body">
        {icon}
        {name}
      </span>
      {status === "connected" ? (
        <DoneBadge>Connected</DoneBadge>
      ) : (
        <Button
          variant="secondary"
          size="sm"
          aria-label={`${action} ${name}`}
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
          {status === "connecting" ? pending : action}
        </Button>
      )}
    </div>
  );
}
