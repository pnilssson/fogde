import { useMutation, useQueryClient } from "@tanstack/react-query";

import { noteQuery, notesQuery } from "@/features/notes/api/notes";
import type { Machine } from "@/lib/machines";
import { report } from "@/lib/report";

export type Entry = "note" | "folder";

function failed(title: string) {
  return (error: Error) => {
    report(title, error);
  };
}

export function useNoteFiles(
  machine: Machine,
  renamed: (from: string, to: string) => void,
) {
  const queries = useQueryClient();
  const refresh = () => queries.invalidateQueries(notesQuery(machine));

  const create = useMutation({
    mutationFn: async (entry: {
      kind: Entry;
      parent: string;
      name: string;
    }) => {
      const method =
        entry.kind === "note" ? "notes.createNote" : "notes.createFolder";
      const path = await machine.client.call(method, {
        parent: entry.parent,
        name: entry.name,
      });
      await refresh();

      return path;
    },
    onError: failed("Could not create it"),
  });

  const rename = useMutation({
    mutationFn: async (entry: { path: string; name: string }) => {
      const path = await machine.client.call("notes.rename", entry);
      const text = queries.getQueryData(
        noteQuery(machine, entry.path).queryKey,
      );
      if (text) {
        queries.setQueryData(noteQuery(machine, path).queryKey, text);
      }
      renamed(entry.path, path);
      await refresh();

      return path;
    },
    onError: failed("Could not rename the note"),
  });

  const remove = useMutation({
    mutationFn: async (path: string) => {
      await machine.client.call("notes.delete", path);
      await refresh();
    },
    onError: failed("Could not delete the note"),
  });

  return { create, rename, remove };
}
