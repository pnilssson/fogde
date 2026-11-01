import type { DragEvent } from "react";

import { beforeOf, changes } from "@/features/workspaces/lib/order";
import { hover, useOver } from "@/features/workspaces/lib/reorder";

export function useReorder(
  id: string,
  siblings: string[],
  moving: string | undefined,
  onArrange: (id: string, before: string | null) => void,
) {
  const over = useOver(id);

  const target = {
    onDragOver: (event: DragEvent<HTMLElement>) => {
      if (!moving) {
        return;
      }

      const row =
        event.currentTarget.querySelector("[data-heading]") ??
        event.currentTarget;
      const box = row.getBoundingClientRect();
      const edge = event.clientY < box.top + box.height / 2 ? "above" : "below";
      const before = beforeOf(siblings, moving, id, edge);
      if (!changes(siblings, moving, before)) {
        hover(undefined);
        return;
      }

      event.preventDefault();
      event.dataTransfer.dropEffect = "move";
      hover({ id, edge });
    },
    onDragLeave: (event: DragEvent<HTMLElement>) => {
      if (
        event.relatedTarget instanceof Node &&
        event.currentTarget.contains(event.relatedTarget)
      ) {
        return;
      }

      hover(undefined);
    },
    onDrop: (event: DragEvent<HTMLElement>) => {
      if (!moving || !over) {
        return;
      }

      event.preventDefault();
      onArrange(moving, beforeOf(siblings, moving, id, over));
      hover(undefined);
    },
  };

  return { over: moving ? over : undefined, target };
}
