import type { ReactNode } from "react";

import { ActionButton } from "@/components/action-button";
import { DoneBadge } from "@/components/done-badge";
import { Button } from "@/components/ui/button";
import type { Status } from "@/lib/connection";

interface Props {
  name: string;
  icon?: ReactNode;
  action?: string;
  pending?: string;
  status: Status;
  onConnect: () => void;
  onCancel?: (() => void) | undefined;
}

export function Connection({
  name,
  icon,
  action = "Connect",
  pending = "Connecting…",
  status,
  onConnect,
  onCancel,
}: Props) {
  const busy = status === "connecting";

  return (
    <div className="flex h-9 w-full items-center justify-between">
      <span className="flex items-center gap-2 type-body">
        {icon}
        {name}
      </span>
      {status === "connected" ? (
        <DoneBadge>Connected</DoneBadge>
      ) : (
        <div className="flex items-center gap-2">
          {busy && onCancel && (
            <Button variant="ghost" size="sm" onClick={onCancel}>
              Cancel
            </Button>
          )}
          <ActionButton
            label={action}
            busy={pending}
            pending={busy}
            aria-label={`${action} ${name}`}
            onClick={onConnect}
          />
        </div>
      )}
    </div>
  );
}
