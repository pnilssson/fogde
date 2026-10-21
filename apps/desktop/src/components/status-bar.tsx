import type { ReactNode } from "react";

import { Separator } from "@/components/ui/separator";

interface Props {
  start: ReactNode;
  folder: string | undefined;
}

export function StatusBar({ start, folder }: Props) {
  return (
    <footer className="shrink-0" aria-label="Workspace status">
      <Separator />
      <div className="flex h-10 items-center gap-4 px-2 type-caption">
        {start}
        {folder && (
          <span className="ml-auto min-w-0 truncate px-2 text-muted-foreground">
            {folder}
          </span>
        )}
      </div>
    </footer>
  );
}
