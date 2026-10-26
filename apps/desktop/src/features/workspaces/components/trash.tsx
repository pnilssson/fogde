import { TrashIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import { type DragEvent, useContext, useState } from "react";

import { CloseTerminal } from "@/features/workspaces/components/close-terminal";
import { useCloseTerminal } from "@/features/workspaces/hooks/use-terminals";
import { useCloseNote } from "@/features/workspaces/hooks/use-workspaces";
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
import { busy } from "@/features/workspaces/lib/status";
import { closeTile } from "@/features/workspaces/lib/opening";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export function Trash({ machine }: { machine: Machine }) {
  const { groups } = useContext(WorkspaceContext);
  const item = useDragging();
  const close = useCloseTerminal(machine);
  const closeNote = useCloseNote();
  const [over, setOver] = useState(false);
  const [target, setTarget] = useState<Terminal>();
  const [confirming, setConfirming] = useState(false);
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

    if (dropped.kind === "note") {
      closeNote.mutate({ machine, tile: dropped.id });
      return;
    }

    const found = findTerminal(groups, dropped);
    if (!found) {
      return;
    }

    if (busy(found.terminal.status)) {
      setTarget(found.terminal);
      setConfirming(true);
      return;
    }

    close.mutate(found.terminal);
  };

  return (
    <>
      {accepts && (
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
      )}
      {target && (
        <CloseTerminal
          machine={machine}
          terminal={target}
          open={confirming}
          onOpenChange={setConfirming}
        />
      )}
    </>
  );
}
