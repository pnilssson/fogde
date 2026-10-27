// @vitest-environment happy-dom
import { createDockview, type DockviewApi } from "dockview-react";
import { afterEach, expect, test, vi } from "vitest";

import { store } from "@/features/workspaces/lib/layouts";
import { canDropNote, dropNote } from "@/features/workspaces/lib/note-drop";
import { openStart, placeTile } from "@/features/workspaces/lib/opening";
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

const a = { kind: "terminal" as const, machine: "m1", id: "a" };

function actions() {
  return { paste: vi.fn(), show: vi.fn(), open: vi.fn() };
}

test("a note dropped on a terminal pastes its path there", () => {
  const api = dockview();
  sync([a], () => false);
  const done = actions();

  dropNote("center", groupOf(api, "m1:a"), done);

  expect(done.paste).toHaveBeenCalledWith({ machine: "m1", id: "a" });
  expect(done.open).not.toHaveBeenCalled();
});

test("a note dropped on an edge opens in a new tile beside it", () => {
  const api = dockview();
  sync([a], () => false);
  const done = actions();
  const group = groupOf(api, "m1:a");

  dropNote("left", group, done);

  expect(done.open).toHaveBeenCalledWith({ position: "left", group });
});

test("a note dropped on a notes tile shows it there", () => {
  const api = dockview();
  sync([{ kind: "note", machine: "m1", id: "note-1" }], () => false);
  const done = actions();

  dropNote("center", groupOf(api, "m1:note-1"), done);

  expect(done.show).toHaveBeenCalledWith({ machine: "m1", id: "note-1" });
});

test("a note dropped on the new tile opens in its place", () => {
  const api = dockview();
  openStart("/repos/app");
  const done = actions();

  dropNote("center", groupOf(api, "start"), done);
  placeTile({ machine: "m1", id: "w1" }, "note", "note-1");

  expect(done.open).toHaveBeenCalledWith(undefined);
  expect(api.getPanel("start")).toBeUndefined();
  expect(api.panels.map(({ id }) => id)).toEqual(["m1:note-1"]);
});

test("a note drops only into a workspace on its own machine", () => {
  const api = dockview();
  sync([a], () => false);

  drag(note);
  expect(canDropNote(groupOf(api, "m1:a"), "center")).toBe(true);

  drag({ machine: "m2", path: "/notes/plan.md" });
  expect(canDropNote(groupOf(api, "m1:a"), "center")).toBe(false);
});
