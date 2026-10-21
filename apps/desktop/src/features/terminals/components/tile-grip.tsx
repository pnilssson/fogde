import { DotsSixIcon } from "@phosphor-icons/react";

import {
  type Dragged,
  dragType,
  endDrag,
  startDrag,
} from "@/features/terminals/lib/drag";

export function TileGrip({ item }: { item: Dragged }) {
  return (
    <div
      aria-hidden
      draggable
      className="absolute top-0 left-1/2 z-10 flex h-4 w-12 -translate-x-1/2 cursor-grab items-center justify-center opacity-0 group-hover/tile:opacity-100 active:cursor-grabbing"
      onDragStart={(event) => {
        event.dataTransfer.effectAllowed = "move";
        event.dataTransfer.setData(dragType, JSON.stringify(item));
        startDrag(item);
      }}
      onDragEnd={endDrag}
    >
      <DotsSixIcon className="size-4" />
    </div>
  );
}
