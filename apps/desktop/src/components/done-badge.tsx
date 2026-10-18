import type { ReactNode } from "react";

import { Badge } from "@/components/ui/badge";

interface Props {
  children: ReactNode;
}

export function DoneBadge({ children }: Props) {
  return (
    <Badge variant="secondary">
      <span aria-hidden className="size-2 rounded-full bg-success" />
      {children}
    </Badge>
  );
}
