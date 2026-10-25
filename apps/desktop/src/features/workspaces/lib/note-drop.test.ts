// @vitest-environment happy-dom
import { createDockview, type DockviewApi } from "dockview-react";
import { afterEach, expect, test, vi } from "vitest";

import { store } from "@/features/workspaces/lib/layouts";
import { canDropNote, dropNote } from "@/features/workspaces/lib/note-drop";
import { openStart } from "@/features/workspaces/lib/opening";
import { attach, sync } from "@/features/workspaces/lib/tiles";
import { noteDragProps } from "@/lib/note-drag";

const note = { machine: "m1", path: "/notes/plan.md" };

let detach: (() => void) | undefined;

function dockview() {
  const element = document.createElement("div");
  document.body.append(element);
  const api = createDockview(element, {
    createComponent: () => ({
      element: document.createElement("div"),
      init: () => undefined,
    }),
  });
  api.layout(900, 600);
  detach = attach(api);
  store.setState({ workspace: { machine: "m1", id: "w1" } });

  return api;
}

function groupOf(api: DockviewApi, id: string) {
  const panel = api.getPanel(id);
  if (!panel) {
    throw new Error(`no tile ${id}`);
  }

  return panel.group;
}

function notes(api: DockviewApi) {
  return api.panels
    .filter((panel) => panel.view.contentComponent === "notes")
    .map(({ params }) => params);
}

function drag(item: { machine: string; path: string }) {
  const data = { setData: () => undefined, effectAllowed: "none" };
  noteDragProps(item).onDragStart({
    dataTransfer: data as unknown as DataTransfer,
  });
}

afterEach(() => {
  noteDragProps(note).onDragEnd();
  detach?.();
  document.body.replaceChildren();
});

test("a note dropped on a terminal pastes its path there", () => {
  const api = dockview();
  sync([{ machine: "m1", id: "a" }], () => false);
  const paste = vi.fn();

  dropNote(note, "center", groupOf(api, "m1:a"), paste);

  expect(paste).toHaveBeenCalledWith({ machine: "m1", id: "a" });
  expect(notes(api)).toEqual([]);
});

test("a note dropped on an edge opens in a new tile beside it", () => {
  const api = dockview();
  sync([{ machine: "m1", id: "a" }], () => false);

  dropNote(note, "left", groupOf(api, "m1:a"), vi.fn());

  expect(notes(api)).toEqual([{ machine: "m1", note: note.path }]);
  expect(api.groups).toHaveLength(2);
});

test("a note dropped on a notes tile or the new tile shows it there", () => {
  const api = dockview();
  openStart("/repos/app");
  dropNote(note, "center", groupOf(api, "start"), vi.fn());
  const [tile] = api.panels;

  expect(api.getPanel("start")).toBeUndefined();
  expect(notes(api)).toEqual([{ machine: "m1", note: note.path }]);

  const other = { machine: "m1", path: "/notes/backlog.md" };
  dropNote(other, "center", tile?.group, vi.fn());

  expect(notes(api)).toEqual([{ machine: "m1", note: other.path }]);
});

test("a note drops only into a workspace on its own machine", () => {
  const api = dockview();
  sync([{ machine: "m1", id: "a" }], () => false);

  drag(note);
  expect(canDropNote(groupOf(api, "m1:a"), "center")).toBe(true);

  drag({ machine: "m2", path: "/notes/plan.md" });
  expect(canDropNote(groupOf(api, "m1:a"), "center")).toBe(false);
});
