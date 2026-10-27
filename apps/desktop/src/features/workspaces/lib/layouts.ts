import type { SerializedDockview } from "dockview-react";
import { z } from "zod";
import { useStore } from "zustand";
import { persist } from "zustand/middleware";
import { createStore } from "zustand/vanilla";

import { type Selection, View } from "@/features/workspaces/lib/selection";

export const startId = "start";
export const holdId = "hold";

interface Tiles {
  workspace: View | undefined;
  layouts: Record<string, SerializedDockview>;
  closed: string[];
  active: Selection | undefined;
  focused: string | undefined;
}

const saved = z.object({
  workspace: View.optional(),
  layouts: z.record(
    z.string(),
    z.custom<SerializedDockview>(
      (layout) => typeof layout === "object" && layout !== null,
    ),
  ),
  closed: z.array(z.string()).default([]),
});

const StartParams = z.object({ folder: z.string() });

const initial: Tiles = {
  workspace: undefined,
  layouts: {},
  closed: [],
  active: undefined,
  focused: undefined,
};

export const store = createStore<Tiles>()(
  persist(() => initial, {
    name: "fogde.tiles",
    partialize: ({ workspace, layouts, closed }) => ({
      workspace,
      layouts,
      closed,
    }),
    merge: (persisted, current) => ({
      ...current,
      ...saved.safeParse(persisted).data,
    }),
  }),
);

export function useTiles<T>(select: (tiles: Tiles) => T) {
  return useStore(store, select);
}

export function layoutKey({ machine, id, checkout }: View) {
  return checkout ? `${machine}/${id}/${checkout}` : `${machine}/${id}`;
}

export function keepMachines(machines: string[]) {
  const { workspace, layouts, closed } = store.getState();

  store.setState({
    closed: closed.filter((key) =>
      machines.some((machine) => key.startsWith(`${machine}/`)),
    ),
    workspace:
      workspace && machines.includes(workspace.machine) ? workspace : undefined,
    layouts: Object.fromEntries(
      Object.entries(layouts).filter(([key]) =>
        machines.some((machine) => key.startsWith(`${machine}/`)),
      ),
    ),
  });
}

export function forget(view: View) {
  const gone = layoutKey(view);
  const kept = (key: string) => key !== gone && !key.startsWith(`${gone}/`);
  const { layouts, closed } = store.getState();

  store.setState({
    closed: closed.filter(kept),
    layouts: Object.fromEntries(
      Object.entries(layouts).filter(([key]) => kept(key)),
    ),
  });
}

type Node = SerializedDockview["grid"]["root"];

function withoutTile(node: Node, id: string): Node {
  if (Array.isArray(node.data)) {
    return { ...node, data: node.data.map((child) => withoutTile(child, id)) };
  }

  const { activeView, ...group } = node.data;
  const views = group.views.filter((view) => view !== id);
  const active = activeView === id ? views[0] : activeView;

  return {
    ...node,
    data: active
      ? { ...group, views, activeView: active }
      : { ...group, views },
  };
}

export function dropTile(view: View, id: string) {
  const key = layoutKey(view);
  const { layouts } = store.getState();
  const layout = layouts[key];
  if (!layout?.panels[id]) {
    return;
  }

  const panels = Object.fromEntries(
    Object.entries(layout.panels).filter(([panel]) => panel !== id),
  );
  const root = withoutTile(layout.grid.root, id);

  store.setState({
    layouts: {
      ...layouts,
      [key]: { ...layout, panels, grid: { ...layout.grid, root } },
    },
  });
}

export function useOpen(view: View) {
  const key = layoutKey(view);

  return useTiles(({ closed }) => !closed.includes(key));
}

export function setOpen(view: View, open: boolean) {
  const key = layoutKey(view);
  const others = store.getState().closed.filter((closed) => closed !== key);

  store.setState({ closed: open ? others : [...others, key] });
}

function sharedStart(layout: SerializedDockview | undefined) {
  if (!layout || Object.keys(layout.panels).length < 2) {
    return undefined;
  }

  return StartParams.safeParse(layout.panels[startId]?.params).data?.folder;
}

export function useStart(view: View | undefined) {
  const layout = useTiles(({ layouts }) => view && layouts[layoutKey(view)]);

  return sharedStart(layout);
}
