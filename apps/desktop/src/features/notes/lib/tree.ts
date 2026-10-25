import type { Note, NoteList } from "@/generated/protocol";
import { folderName } from "@/lib/folders";

export const orders = [
  "name-asc",
  "name-desc",
  "modified-desc",
  "modified-asc",
  "created-desc",
  "created-asc",
] as const;

export type Order = (typeof orders)[number];

export interface Folder {
  name: string;
  path: string;
  folders: Folder[];
  notes: Note[];
}

const names = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

function trimmed(path: string) {
  return path.replace(/(?<=.)[\\/]+$/, "");
}

function parentOf(path: string) {
  const at = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));

  return at > 0 ? path.slice(0, at) : path;
}

function compareNotes(order: Order) {
  return (a: Note, b: Note) => {
    const byName = names.compare(a.name, b.name);
    switch (order) {
      case "name-asc":
        return byName;
      case "name-desc":
        return -byName;
      case "modified-desc":
        return b.modified - a.modified || byName;
      case "modified-asc":
        return a.modified - b.modified || byName;
      case "created-desc":
        return b.created - a.created || byName;
      case "created-asc":
        return a.created - b.created || byName;
    }
  };
}

function sort(folder: Folder, order: Order) {
  const direction = order === "name-desc" ? -1 : 1;
  folder.folders.sort((a, b) => direction * names.compare(a.name, b.name));
  folder.notes.sort(compareNotes(order));
  for (const inner of folder.folders) {
    sort(inner, order);
  }
}

export function buildTree(list: NoteList, order: Order): Folder {
  const root: Folder = {
    name: folderName(list.root),
    path: list.root,
    folders: [],
    notes: [],
  };
  const byPath = new Map([[trimmed(list.root), root]]);

  for (const path of list.folders) {
    byPath.set(trimmed(path), {
      name: folderName(path),
      path,
      folders: [],
      notes: [],
    });
  }
  for (const path of list.folders) {
    const folder = byPath.get(trimmed(path));
    const parent = byPath.get(trimmed(parentOf(trimmed(path))));
    if (folder && parent && folder !== parent) {
      parent.folders.push(folder);
    }
  }
  for (const note of list.notes) {
    byPath.get(trimmed(note.folder))?.notes.push(note);
  }

  sort(root, order);

  return root;
}

export function foldersAbove(path: string, root: string) {
  const top = trimmed(root);
  const above: string[] = [];
  let current = trimmed(parentOf(path));

  while (current.length > top.length) {
    above.push(current);
    const next = trimmed(parentOf(current));
    if (next === current) {
      break;
    }
    current = next;
  }

  return above;
}
