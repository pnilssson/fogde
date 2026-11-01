import { cn } from "cn";
import { useState } from "react";

import type { Status } from "@/generated/protocol";

interface Props {
  status: Status;
  className?: string | undefined;
}

interface Look {
  label: string;
  className: string;
}

const breath = 2000;

const dots: Partial<Record<Status, Look>> = {
  working: {
    label: "Working",
    className: "animate-breathe bg-info motion-reduce:animate-none",
  },
  blocked: { label: "Needs your input", className: "bg-destructive" },
  done: { label: "Finished", className: "bg-success" },
};

export function StateIcon({ status, className }: Props) {
  const dot = dots[status];
  if (!dot) {
    return null;
  }

  return <Dot key={status} look={dot} className={className} />;
}

function Dot({
  look,
  className,
}: {
  look: Look;
  className: string | undefined;
}) {
  const [delay] = useState(
    () => `-${String(Math.round(performance.now() % breath))}ms`,
  );

  return (
    <span
      role="img"
      aria-label={look.label}
      style={{ animationDelay: delay }}
      className={cn("size-2 shrink-0 rounded-full", look.className, className)}
    />
  );
}
