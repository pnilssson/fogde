import type { DockviewApi, DockviewPanelApi } from "dockview-react";

import {
  dropTile,
  type NotesParams,
  startId,
  store,
} from "@/features/workspaces/lib/layouts";
import {
  sameView,
  type Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import { requestStartFocus } from "@/features/workspaces/lib/start-focus";
import {
  dockview,
  saveLayout,
  show,
  tileId,
} from "@/features/workspaces/lib/tiles";

function beside(api: DockviewApi) {
  const focused = api.activePanel;

  return focused
    ? { referencePanel: focused, direction: "right" as const }
    : { direction: "right" as const };
}

export function openStart(folder: string, focus = false) {
  const api = dockview();
  if (!api) {
    return;
  }

  requestStartFocus(focus);

  const existing = api.getPanel(startId);
  if (existing) {
    existing.api.updateParameters({ folder });
    existing.api.setActive();
    return;
  }

  api.addPanel({
    id: startId,
    component: "start",
    params: { folder },
    position: beside(api),
  });
}

export function closeTile(id: string, view?: View) {
  const api = dockview();
  if (api && (!view || sameView(view, store.getState().workspace))) {
    api.getPanel(id)?.api.close();
    return;
  }

  if (view) {
    dropTile(view, id);
  }
}

export function closeStart(view?: View) {
  closeTile(startId, view);
}

export function opened(selection: Selection) {
  const api = dockview();
  const start = api?.getPanel(startId);
  if (!api || !start) {
    show(selection);
    return;
  }

  const existing = api.getPanel(tileId(selection));
  if (existing) {
    existing.api.moveTo({ group: start.group, position: "center" });
    start.api.close();
    return;
  }

  api.addPanel({
    id: tileId(selection),
    component: "terminal",
    params: selection,
    position: { referenceGroup: start.group, direction: "within" },
  });
  start.api.close();
}

export function notesPanel(machine: string, note?: string) {
  const params: NotesParams = note ? { machine, note } : { machine };

  return { id: `notes:${crypto.randomUUID()}`, component: "notes", params };
}

export function openNotes(machine: string, note?: string) {
  const api = dockview();
  if (!api) {
    return;
  }

  const notes = notesPanel(machine, note);
  const start = api.getPanel(startId);
  if (!start) {
    api.addPanel({ ...notes, position: beside(api) });
    return;
  }

  api.addPanel({
    ...notes,
    position: { referenceGroup: start.group, direction: "within" },
  });
  start.api.close();
}

export function showNote(panel: DockviewPanelApi, params: NotesParams) {
  panel.updateParameters(params);
  saveLayout();
}
