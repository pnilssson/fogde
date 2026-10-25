import { useMutation, useQueryClient } from "@tanstack/react-query";

import { noteQuery, notesQuery } from "@/features/notes/api/notes";
import { follow } from "@/features/notes/lib/recent";
import type { NoteText } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { announce, moved } from "@/lib/note-renames";
import { report } from "@/lib/report";

export type Entry = "note" | "folder";

function failed(title: string) {
  return (error: Error) => {
    report(title, error);
  };
}

export function useNoteFiles(machine: Machine) {
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
      const cached = queries.getQueriesData<NoteText>({
        queryKey: [machine.id, "notes.read"],
      });
      for (const [[, , old], text] of cached) {
        const next =
          typeof old === "string" ? moved(old, entry.path, path) : undefined;
        if (next && text) {
          queries.setQueryData(noteQuery(machine, next).queryKey, text);
        }
      }

      const rename = { machine: machine.id, from: entry.path, to: path };
      follow(rename);
      announce(rename);
      await refresh();

      return path;
    },
    onError: failed("Could not rename it"),
  });

  const remove = useMutation({
    mutationFn: async (path: string) => {
      await machine.client.call("notes.delete", path);
      await refresh();
    },
    onError: failed("Could not delete it"),
  });

  return { create, rename, remove };
}
