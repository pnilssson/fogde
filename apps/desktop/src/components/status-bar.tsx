import type { ReactNode } from "react";

import { Separator } from "@/components/ui/separator";

interface Props {
  start: ReactNode;
}

export function StatusBar({ start }: Props) {
  return (
    <footer className="shrink-0" aria-label="Status">
      <Separator />
      <div className="flex h-10 items-center gap-4 px-2 type-caption">
        {start}
      </div>
    </footer>
  );
}
