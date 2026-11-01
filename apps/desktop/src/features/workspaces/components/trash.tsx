import { TrashIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import { type DragEvent, useContext, useState } from "react";

import { useCloseRequest } from "@/features/workspaces/hooks/use-close";
import {
  type Dragged,
  dragged,
  endDrag,
  isListed,
  useDragging,
} from "@/features/workspaces/lib/drag";
import {
  findTerminal,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import { closeTile } from "@/features/workspaces/lib/opening";
import type { Machine } from "@/lib/machines";

export function Trash({ machine }: { machine: Machine }) {
  const { groups } = useContext(WorkspaceContext);
  const item = useDragging();
  const close = useCloseRequest(machine);
  const [over, setOver] = useState(false);
  const accepts =
    item !== undefined && (!isListed(item) || item.machine === machine.id);

  const hover = (event: DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
    setOver(true);
  };

  const discard = (dropped: Dragged) => {
    if (!isListed(dropped)) {
      closeTile(dropped.tile, dropped.view);
      return;
    }

    const found = findTerminal(groups, dropped);
    if (found) {
      close(found.terminal);
    }
  };

  if (!accepts) {
    return null;
  }

  return (
    <div
      aria-hidden
      className={cn(
        "absolute bottom-4 left-1/2 z-20 flex size-12 -translate-x-1/2 animate-in items-center justify-center rounded-full bg-popover shadow-surface-5 duration-150 ease-out fade-in-0 motion-safe:zoom-in-95",
        over && "text-destructive",
      )}
      onDragEnter={hover}
      onDragOver={hover}
      onDragLeave={() => {
        setOver(false);
      }}
      onDrop={(event) => {
        event.preventDefault();
        setOver(false);
        const dropped = dragged(event.nativeEvent);
        endDrag();
        if (dropped) {
          discard(dropped);
        }
      }}
    >
      <TrashIcon
        weight={over ? "fill" : "regular"}
        className="pointer-events-none size-6"
      />
    </div>
  );
}
