import { cn } from "cn";

import { Spinner } from "@/components/ui/spinner";
import type { Status } from "@/generated/protocol";

interface Props {
  status: Status;
  className?: string | undefined;
}

const dots: Partial<Record<Status, { label: string; className: string }>> = {
  blocked: { label: "Needs your input", className: "bg-destructive" },
  done: { label: "Finished", className: "bg-success" },
};

export function StateIcon({ status, className }: Props) {
  if (status === "working") {
    return (
      <Spinner
        aria-label="Working"
        className={cn("size-3 shrink-0 text-muted-foreground", className)}
      />
    );
  }

  const dot = dots[status];
  if (!dot) {
    return null;
  }

  return (
    <span
      role="img"
      aria-label={dot.label}
      className={cn("size-2 shrink-0 rounded-full", dot.className, className)}
    />
  );
}
