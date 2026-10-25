import { expect, test } from "vitest";

import {
  buildTree,
  type Folder,
  foldersAbove,
} from "@/features/notes/lib/tree";
import type { Note, NoteList } from "@/generated/protocol";

function note(folder: string, name: string, modified: number): Note {
  return {
    name,
    path: `${folder}\\${name}.md`,
    folder,
    modified,
    created: modified,
  };
}

const root = "C:\\notes";
const list: NoteList = {
  root,
  folders: [`${root}\\app`, `${root}\\app\\specs`, `${root}\\Web`],
  notes: [
    note(root, "note 10", 1),
    note(root, "Note 2", 3),
    note(root, "backlog", 2),
    note(`${root}\\app`, "plan", 1),
    note(`${root}\\app\\specs`, "login", 1),
  ],
};

function shape(folder: Folder): unknown {
  return {
    name: folder.name,
    folders: folder.folders.map(shape),
    notes: folder.notes.map(({ name }) => name),
  };
}

test("builds the folder tree sorted by name like Obsidian", () => {
  expect(shape(buildTree(list, "name-asc"))).toEqual({
    name: "notes",
    folders: [
      {
        name: "app",
        folders: [{ name: "specs", folders: [], notes: ["login"] }],
        notes: ["plan"],
      },
      { name: "Web", folders: [], notes: [] },
    ],
    notes: ["backlog", "Note 2", "note 10"],
  });
});

test("sorts notes by time while folders stay by name", () => {
  const tree = buildTree(list, "modified-desc");

  expect(tree.notes.map(({ name }) => name)).toEqual([
    "Note 2",
    "backlog",
    "note 10",
  ]);
  expect(tree.folders.map(({ name }) => name)).toEqual(["app", "Web"]);
  expect(buildTree(list, "name-desc").folders.map(({ name }) => name)).toEqual([
    "Web",
    "app",
  ]);
});

test("lists the folders above a note up to the root", () => {
  expect(foldersAbove(`${root}\\app\\specs\\login.md`, root)).toEqual([
    `${root}\\app\\specs`,
    `${root}\\app`,
  ]);
  expect(foldersAbove("/notes/plan.md", "/notes/")).toEqual([]);
});
