import type { SerializedDockview } from "dockview-react";
import { z } from "zod";
import { useStore } from "zustand";
import { persist } from "zustand/middleware";
import { createStore } from "zustand/vanilla";

import { Selection } from "@/features/terminals/lib/selection";

export const startId = "start";
export const holdId = "hold";

interface Tiles {
  workspace: Selection | undefined;
  layouts: Record<string, SerializedDockview>;
  closed: string[];
  active: Selection | undefined;
  startActive: boolean;
}

const saved = z.object({
  workspace: Selection.optional(),
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
  startActive: false,
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

export function layoutKey({ machine, id }: Selection) {
  return `${machine}/${id}`;
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

export function forget(workspace: Selection) {
  const gone = layoutKey(workspace);
  const { layouts, closed } = store.getState();

  store.setState({
    closed: closed.filter((key) => key !== gone),
    layouts: Object.fromEntries(
      Object.entries(layouts).filter(([key]) => key !== gone),
    ),
  });
}

export function useOpen(workspace: Selection) {
  const key = layoutKey(workspace);

  return useTiles(({ closed }) => !closed.includes(key));
}

export function setOpen(workspace: Selection, open: boolean) {
  const key = layoutKey(workspace);
  const others = store.getState().closed.filter((closed) => closed !== key);

  store.setState({ closed: open ? others : [...others, key] });
}

function sharedStart(layout: SerializedDockview | undefined) {
  if (!layout || Object.keys(layout.panels).length < 2) {
    return undefined;
  }

  return StartParams.safeParse(layout.panels[startId]?.params).data?.folder;
}

export function useStart(workspace: Selection | undefined) {
  const layout = useTiles(
    ({ layouts }) => workspace && layouts[layoutKey(workspace)],
  );

  return sharedStart(layout);
}
