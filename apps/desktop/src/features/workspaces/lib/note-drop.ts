import type { DockviewGroupPanel, Position } from "dockview-react";

import { store } from "@/features/workspaces/lib/layouts";
import type { Spot } from "@/features/workspaces/lib/opening";
import { Selection } from "@/features/workspaces/lib/selection";
import { dockview } from "@/features/workspaces/lib/tiles";
import { draggingNote } from "@/lib/note-drag";

export function canDropNote(
  group: DockviewGroupPanel | undefined,
  position: Position,
) {
  const api = dockview();
  const note = draggingNote();
  if (!api || note?.machine !== store.getState().workspace?.machine) {
    return false;
  }

  return (
    group !== undefined || position !== "center" || api.panels.length === 0
  );
}

interface Actions {
  paste: (terminal: Selection) => void;
  show: (tile: Selection) => void;
  open: (spot: Spot | undefined) => void;
}

export function dropNote(
  position: Position,
  group: DockviewGroupPanel | undefined,
  { paste, show, open }: Actions,
) {
  const panel = group?.activePanel;
  if (panel && position === "center") {
    const tile = Selection.safeParse(panel.params).data;
    switch (panel.view.contentComponent) {
      case "terminal":
        if (tile) {
          paste(tile);
          panel.focus();
        }
        return;
      case "notes":
        if (tile) {
          show(tile);
          panel.focus();
        }
        return;
      case "start":
        open(undefined);
        return;
    }
  }

  open({ position, group });
}
