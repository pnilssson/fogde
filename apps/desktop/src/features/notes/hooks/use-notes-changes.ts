import { useQueryClient } from "@tanstack/react-query";

import { noteQuery, notesQuery } from "@/features/notes/api/notes";
import type { Note, NoteList } from "@/generated/protocol";
import { useMachineEvent } from "@/hooks/use-machine-event";
import type { Machine } from "@/lib/machines";

function patched(list: NoteList, notes: Note[]): NoteList | undefined {
  const fresh = new Map(notes.map((note) => [note.path, note]));
  if (notes.some((note) => !list.notes.some((old) => old.path === note.path))) {
    return undefined;
  }

  return {
    ...list,
    notes: list.notes.map((note) => fresh.get(note.path) ?? note),
  };
}

export function useNotesChanges(machines: Machine[]) {
  const queries = useQueryClient();

  useMachineEvent(machines, "notes.changed", (machine, changed) => {
    const list = notesQuery(machine).queryKey;
    if (changed.listing) {
      void queries.invalidateQueries({ queryKey: list });
      void queries.invalidateQueries({
        queryKey: [machine.id, "notes.read"],
      });
      return;
    }

    const current = queries.getQueryData(list);
    const next = current && patched(current, changed.notes);
    if (next) {
      queries.setQueryData(list, next);
    } else {
      void queries.invalidateQueries({ queryKey: list });
    }
    for (const note of changed.notes) {
      void queries.invalidateQueries({
        queryKey: noteQuery(machine, note.path).queryKey,
      });
    }
  });
}
