import type { SerializedDockview } from "dockview-react";
import { z } from "zod";
import { useStore } from "zustand";
import { persist } from "zustand/middleware";
import { createStore } from "zustand/vanilla";

import { type Selection, View } from "@/features/workspaces/lib/selection";
import { folderName } from "@/lib/folders";
import { moved, type Rename } from "@/lib/note-renames";

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

export const NotesParams = z.object({
  machine: z.string(),
  note: z.string().optional(),
});

export type NotesParams = z.infer<typeof NotesParams>;

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

export function followedNote(params: unknown, rename: Rename) {
  const parsed = NotesParams.safeParse(params).data;
  const note =
    parsed?.note && parsed.machine === rename.machine
      ? moved(parsed.note, rename.from, rename.to)
      : undefined;

  return parsed && note ? { ...parsed, note } : undefined;
}

export function followNotes(rename: Rename) {
  const { layouts } = store.getState();
  const follow = (layout: SerializedDockview) => ({
    ...layout,
    panels: Object.fromEntries(
      Object.entries(layout.panels).map(([id, panel]) => {
        const params =
          panel.contentComponent === "notes"
            ? followedNote(panel.params, rename)
            : undefined;

        return [id, params ? { ...panel, params } : panel];
      }),
    ),
  });

  store.setState({
    layouts: Object.fromEntries(
      Object.entries(layouts).map(([key, layout]) => [key, follow(layout)]),
    ),
  });
}

export function noteTitle(note: string | undefined) {
  return note ? folderName(note).replace(/\.md$/i, "") : "Notes";
}

function noteTiles(layout: SerializedDockview | undefined) {
  return Object.values(layout?.panels ?? {}).flatMap((panel) => {
    const params = NotesParams.safeParse(panel.params).data;

    return panel.contentComponent === "notes" && params
      ? [{ id: panel.id, title: noteTitle(params.note) }]
      : [];
  });
}

export function useNoteTiles(view: View) {
  const layout = useTiles(({ layouts }) => layouts[layoutKey(view)]);

  return noteTiles(layout);
}
