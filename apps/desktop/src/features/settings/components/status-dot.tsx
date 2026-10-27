import { cn } from "cn";

import {
  type Status,
  statusColors,
  statusLabels,
} from "@/features/settings/hooks/use-status";

export function StatusDot({
  status,
  labelled = false,
}: {
  status: Status;
  labelled?: boolean;
}) {
  return (
    <span
      role={labelled ? "img" : undefined}
      aria-label={labelled ? statusLabels[status] : undefined}
      aria-hidden={!labelled}
      className={cn("size-2 shrink-0 rounded-full", statusColors[status])}
    />
  );
}
