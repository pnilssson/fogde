import { FileTextIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useCommandState } from "cmdk";

import { CommandGroup, CommandItem } from "@/components/ui/command";
import { notesQuery } from "@/features/notes/api/notes";
import { recent, useViewed } from "@/features/notes/lib/recent";
import type { Note, NoteList } from "@/generated/protocol";
import { relative } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

const found = 20;

interface Props {
  machine: Machine;
  onOpen: (note: string) => void;
}

interface Match {
  note: Note;
  name: string;
  folder: string;
}

function rank({ name }: Match, search: string) {
  if (name.startsWith(search)) {
    return 0;
  }

  return name.includes(search) ? 1 : 2;
}

function matching({ notes, root }: NoteList, search: string) {
  const query = search.trim().toLowerCase();
  const words = query.split(/\s+/);

  return notes
    .map((note) => ({
      note,
      name: note.name.toLowerCase(),
      folder: (relative(note.folder, root) ?? "").toLowerCase(),
    }))
    .filter(({ name, folder }) =>
      words.every((word) => `${name} ${folder}`.includes(word)),
    )
    .sort(
      (a, b) =>
        rank(a, query) - rank(b, query) || b.note.modified - a.note.modified,
    )
    .slice(0, found)
    .map(({ note }) => note);
}

export function PaletteNotes({ machine, onOpen }: Props) {
  const { data } = useQuery(notesQuery(machine));
  const viewed = useViewed(machine.id);
  const search = useCommandState((state) => state.search);
  if (!data || data.notes.length === 0) {
    return null;
  }

  const notes = search.trim()
    ? matching(data, search)
    : recent(data.notes, viewed);

  return (
    <CommandGroup heading={search.trim() ? "Notes" : "Recent notes"}>
      {notes.map((note) => {
        const folder = relative(note.folder, data.root);

        return (
          <CommandItem
            key={note.path}
            value={`note ${note.path}`}
            keywords={folder ? [note.name, folder] : [note.name]}
            onSelect={() => {
              onOpen(note.path);
            }}
          >
            <FileTextIcon />
            <span className="flex min-w-0 gap-2">
              <span className="truncate">{note.name}</span>
              {folder && (
                <span className="truncate text-muted-foreground">{folder}</span>
              )}
            </span>
          </CommandItem>
        );
      })}
    </CommandGroup>
  );
}
