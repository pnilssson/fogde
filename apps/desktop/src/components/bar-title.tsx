import { cn } from "cn";
import type { ReactNode } from "react";

interface Props {
  icon?: ReactNode;
  title: string;
  muted?: boolean;
}

export function BarTitle({ icon, title, muted = false }: Props) {
  return (
    <span
      className={cn(
        "flex min-w-0 items-center gap-2 font-mono type-caption",
        muted && "text-muted-foreground",
      )}
    >
      {icon}
      <span className="truncate">{title}</span>
    </span>
  );
}
