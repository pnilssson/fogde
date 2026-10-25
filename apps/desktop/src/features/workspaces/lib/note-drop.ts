import type { DockviewGroupPanel, Position } from "dockview-react";
import { positionToDirection } from "dockview-react";

import { store } from "@/features/workspaces/lib/layouts";
import {
  notesPanel,
  openNotes,
  showNote,
} from "@/features/workspaces/lib/opening";
import { Selection } from "@/features/workspaces/lib/selection";
import { dockview } from "@/features/workspaces/lib/tiles";
import { draggingNote, type NoteDrag } from "@/lib/note-drag";

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

export function dropNote(
  note: NoteDrag,
  position: Position,
  group: DockviewGroupPanel | undefined,
  paste: (terminal: Selection) => void,
) {
  const api = dockview();
  if (!api) {
    return;
  }

  const panel = group?.activePanel;
  if (panel && position === "center") {
    switch (panel.view.contentComponent) {
      case "terminal": {
        const terminal = Selection.safeParse(panel.params).data;
        if (terminal) {
          paste(terminal);
          panel.focus();
        }
        return;
      }
      case "notes":
        showNote(panel.api, { machine: note.machine, note: note.path });
        panel.focus();
        return;
      case "start":
        openNotes(note.machine, note.path);
        return;
    }
  }

  const direction =
    position === "center" ? "right" : positionToDirection(position);
  api.addPanel({
    ...notesPanel(note.machine, note.path),
    position: group ? { referenceGroup: group, direction } : { direction },
  });
}
