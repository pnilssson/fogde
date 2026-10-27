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

import { holdId, store } from "@/features/workspaces/lib/layouts";
import { Selection, View } from "@/features/workspaces/lib/selection";
import { dockview, place, tileId } from "@/features/workspaces/lib/tiles";

export const dragType = "application/x-fogde-tile";

const Placed = z.object({ tile: z.string(), view: View.optional() });
const Dragged = z.union([Selection, Placed]);
export type Dragged = z.infer<typeof Dragged>;

export type Edge = "above" | "below";

interface Over {
  id: string;
  edge: Edge;
}

const drags = createStore<{
  item: Dragged | undefined;
  over: Over | undefined;
}>()(() => ({
  item: undefined,
  over: undefined,
}));

export function useDragging() {
  return useStore(drags, ({ item }) => item);
}

export function useOver(id: string) {
  return useStore(drags, ({ over }) =>
    over?.id === id ? over.edge : undefined,
  );
}

export function hover(over: Over | undefined) {
  const current = drags.getState().over;
  if (current?.id === over?.id && current?.edge === over?.edge) {
    return;
  }

  drags.setState({ over });
}

export function isListed(item: Dragged): item is Selection {
  return "id" in item;
}

function panelId(item: Dragged) {
  return isListed(item) ? tileId(item) : item.tile;
}

export function dragProps(item: Dragged) {
  return {
    draggable: true,
    onDragStart: (event: { dataTransfer: DataTransfer }) => {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData(dragType, JSON.stringify(item));
      startDrag(item);
    },
    onDragEnd: endDrag,
  };
}

export function startDrag(item: Dragged) {
  drags.setState({ item });
}

export function endDrag() {
  drags.setState({ item: undefined, over: undefined });
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

export type Side = Exclude<Position, "center">;

function gap(from: DOMRect, to: DOMRect, side: Side) {
  switch (side) {
    case "left":
      return from.left - to.right;
    case "right":
      return to.left - from.right;
    case "top":
      return from.top - to.bottom;
    case "bottom":
      return to.top - from.bottom;
  }
}

function overlap(from: DOMRect, to: DOMRect, side: Side) {
  return side === "left" || side === "right"
    ? Math.min(from.bottom, to.bottom) - Math.max(from.top, to.top)
    : Math.min(from.right, to.right) - Math.max(from.left, to.left);
}

function neighbour(api: DockviewApi, home: DockviewGroupPanel, side: Side) {
  const from = home.element.getBoundingClientRect();
  let best:
    { group: DockviewGroupPanel; gap: number; overlap: number } | undefined;

  for (const group of api.groups) {
    const to = group.element.getBoundingClientRect();
    const near = {
      group,
      gap: gap(from, to, side),
      overlap: overlap(from, to, side),
    };
    if (group === home || near.gap < -1 || near.overlap <= 0) {
      continue;
    }
    if (
      !best ||
      near.gap < best.gap ||
      (near.gap === best.gap && near.overlap > best.overlap)
    ) {
      best = near;
    }
  }

  return best?.group;
}

export function moveTile(side: Side) {
  const api = dockview();
  const panel = api?.activePanel;
  if (!api || !panel || api.groups.length < 2) {
    return;
  }

  const group = neighbour(api, panel.group, side);
  if (group) {
    swap(api, panel, group);
  } else {
    edge(api, panel, side);
  }
  panel.focus();
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

  const terminal = isListed(dragging);
  if (terminal && dragging.machine !== store.getState().workspace?.machine) {
    return false;
  }

  const existing = api.getPanel(panelId(dragging));
  if (!existing) {
    return terminal && (position !== "center" || api.panels.length === 0);
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
    if (isListed(item)) {
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
