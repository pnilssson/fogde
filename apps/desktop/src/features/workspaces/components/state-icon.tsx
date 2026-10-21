import { cn } from "cn";

import type { Status } from "@/generated/protocol";

interface Props {
  status: Status;
  className?: string | undefined;
}

const states: Partial<Record<Status, { label: string; className: string }>> = {
  working: {
    label: "Working",
    className: "bg-warning",
  },
  blocked: { label: "Needs your input", className: "bg-destructive" },
  done: { label: "Finished", className: "bg-success" },
};

export function StateIcon({ status, className }: Props) {
  const state = states[status];
  if (!state) {
    return null;
  }

  return (
    <span
      role="img"
      aria-label={state.label}
      className={cn("size-2 shrink-0 rounded-full", state.className, className)}
    />
  );
}
