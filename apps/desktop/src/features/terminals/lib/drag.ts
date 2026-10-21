import type {
  DockviewApi,
  DockviewGroupPanel,
  IDockviewPanel,
  Position,
} from "dockview-react";
import { positionToDirection } from "dockview-react";
import { z } from "zod";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import { holdId, startId, store } from "@/features/terminals/lib/layouts";
import { Selection } from "@/features/terminals/lib/selection";
import { dockview, place, tileId } from "@/features/terminals/lib/tiles";

export const dragType = "application/x-fogde-terminal";

const Dragged = z.union([Selection, z.literal(startId)]);
export type Dragged = z.infer<typeof Dragged>;

const drags = createStore<{ item: Dragged | undefined }>()(() => ({
  item: undefined,
}));

export function useDragging() {
  return useStore(drags, ({ item }) => item);
}

function panelId(item: Dragged) {
  return item === startId ? startId : tileId(item);
}

export function startDrag(item: Dragged) {
  drags.setState({ item });
}

export function endDrag() {
  drags.setState({ item: undefined });
}

export function dragged(event: DragEvent) {
  try {
    const data = event.dataTransfer?.getData(dragType);

    return data ? Dragged.safeParse(JSON.parse(data)).data : undefined;
  } catch {
    return undefined;
  }
}

function swap(
  api: DockviewApi,
  panel: IDockviewPanel,
  group: DockviewGroupPanel,
) {
  const home = panel.group;
  const other = group.activePanel;
  if (!other || home === group) {
    return;
  }

  const hold = api.addPanel({
    id: holdId,
    component: holdId,
    position: { referenceGroup: home, direction: "within" },
  });
  panel.api.moveTo({ group, position: "center" });
  other.api.moveTo({ group: home, position: "center" });
  hold.api.close();
  panel.api.setActive();
}

function edge(api: DockviewApi, panel: IDockviewPanel, position: Position) {
  if (position === "center") {
    return;
  }

  const group = api.addGroup({ direction: positionToDirection(position) });
  panel.api.moveTo({ group, position: "center" });
}

export function canDrop(
  group: DockviewGroupPanel | undefined,
  position: Position,
) {
  const api = dockview();
  const dragging = drags.getState().item;
  if (!api || !dragging) {
    return false;
  }

  if (
    dragging !== startId &&
    dragging.machine !== store.getState().workspace?.machine
  ) {
    return false;
  }

  const existing = api.getPanel(panelId(dragging));
  if (!existing) {
    return (
      dragging !== startId && (position !== "center" || api.panels.length === 0)
    );
  }

  if (!group) {
    return position !== "center" && api.groups.length > 1;
  }

  return group !== existing.group;
}

export function drop(
  item: Dragged,
  position: Position,
  group: DockviewGroupPanel | undefined,
) {
  const api = dockview();
  if (!api) {
    return;
  }

  const existing = api.getPanel(panelId(item));
  if (!existing) {
    if (item !== startId) {
      place(api, item, position, group);
    }

    return;
  }

  if (!group) {
    edge(api, existing, position);
    return;
  }

  if (position === "center") {
    swap(api, existing, group);
    return;
  }

  existing.api.moveTo({ group, position });
}
