import type { DockviewApi, DockviewGroupPanel, Position } from "dockview-react";

import type { Listed } from "@/features/workspaces/lib/drag";
import { dropTile, startId, store } from "@/features/workspaces/lib/layouts";
import {
  sameView,
  type Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import { requestStartFocus } from "@/features/workspaces/lib/start-focus";
import {
  dockview,
  place,
  show,
  tileId,
  tilePanel,
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

export interface Spot {
  position: Position;
  group: DockviewGroupPanel | undefined;
}

export function placeTile(
  view: View,
  kind: Listed["kind"],
  id: string,
  spot?: Spot,
) {
  const api = dockview();
  const tile = { kind, machine: view.machine, id };
  if (!api || !sameView(view, store.getState().workspace)) {
    return;
  }

  const existing = api.getPanel(tileId(tile));
  if (existing) {
    existing.api.setActive();
    return;
  }

  if (spot) {
    const group = api.groups.find((current) => current === spot.group);
    place(api, tile, spot.position, group);
    return;
  }

  const start = api.getPanel(startId);
  if (!start) {
    api.addPanel({ ...tilePanel(tile), position: beside(api) });
    return;
  }

  api.addPanel({
    ...tilePanel(tile),
    position: { referenceGroup: start.group, direction: "within" },
  });
  start.api.close();
}
