import { z } from "zod";
import { useStore } from "zustand";
import { persist } from "zustand/middleware";
import { createStore } from "zustand/vanilla";

import type { Note } from "@/generated/protocol";
import { moved, type Rename } from "@/lib/note-renames";

const kept = 20;
const shown = 8;

const saved = z.object({ viewed: z.record(z.string(), z.array(z.string())) });

type Viewed = Record<string, string[]>;

const store = createStore<{ viewed: Viewed }>()(
  persist((): { viewed: Viewed } => ({ viewed: {} }), {
    name: "fogde.notes.recent",
    merge: (persisted, current) => ({
      ...current,
      ...saved.safeParse(persisted).data,
    }),
  }),
);

export function useViewed(machine: string) {
  return useStore(store, ({ viewed }) => viewed[machine]);
}

export function view(machine: string, path: string) {
  store.setState(({ viewed }) => ({
    viewed: {
      ...viewed,
      [machine]: [
        path,
        ...(viewed[machine] ?? []).filter((item) => item !== path),
      ].slice(0, kept),
    },
  }));
}

export function follow({ machine, from, to }: Rename) {
  store.setState(({ viewed }) => ({
    viewed: {
      ...viewed,
      [machine]: (viewed[machine] ?? []).map(
        (path) => moved(path, from, to) ?? path,
      ),
    },
  }));
}

export function recent(notes: Note[], viewed: string[] = []) {
  const byPath = new Map(notes.map((note) => [note.path, note]));
  const seen = viewed.flatMap((path) => byPath.get(path) ?? []);
  if (seen.length >= shown) {
    return seen.slice(0, shown);
  }

  const rest = notes
    .filter((note) => !seen.includes(note))
    .sort((a, b) => b.modified - a.modified);

  return [...seen, ...rest].slice(0, shown);
}
