import { cn } from "cn";
import type { ReactNode } from "react";

interface Props {
  icon?: ReactNode;
  title: string;
  muted?: boolean;
  mono?: boolean;
}

export function BarTitle({ icon, title, muted = false, mono = false }: Props) {
  return (
    <span
      className={cn(
        "flex min-w-0 items-center gap-2",
        mono ? "font-mono type-caption" : "type-body",
        muted && "text-muted-foreground",
      )}
    >
      {icon}
      <span className="truncate">{title}</span>
    </span>
  );
}
