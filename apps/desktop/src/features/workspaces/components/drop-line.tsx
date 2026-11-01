import { cn } from "cn";

import type { Edge } from "@/features/workspaces/lib/reorder";

export function DropLine({ edge }: { edge: Edge | undefined }) {
  if (!edge) {
    return null;
  }

  return (
    <span
      aria-hidden
      className={cn(
        "pointer-events-none absolute inset-x-0 h-0.5 rounded-full bg-sidebar-ring",
        edge === "above"
          ? "-top-0.5 -translate-y-1/2"
          : "-bottom-0.5 translate-y-1/2",
      )}
    />
  );
}
