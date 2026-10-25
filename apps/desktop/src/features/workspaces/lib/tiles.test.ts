// @vitest-environment happy-dom
import { createDockview, type DockviewApi } from "dockview-react";
import { afterEach, expect, test } from "vitest";

import {
  canDrop,
  drop,
  endDrag,
  startDrag,
} from "@/features/workspaces/lib/drag";
import { layoutKey, store } from "@/features/workspaces/lib/layouts";
import { closeTile } from "@/features/workspaces/lib/opening";
import {
  activate,
  attach,
  selectWorkspace,
  sync,
} from "@/features/workspaces/lib/tiles";

const a = { machine: "m1", id: "a" };
const b = { machine: "m1", id: "b" };
const c = { machine: "m1", id: "c" };

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

  return api;
}

function tiles(api: DockviewApi) {
  return api.groups.map((group) => group.panels.map(({ id }) => id));
}

function groupOf(api: DockviewApi, id: string) {
  const panel = api.getPanel(id);
  if (!panel) {
    throw new Error(`no tile ${id}`);
  }

  return panel.group;
}

afterEach(() => {
  detach?.();
  document.body.replaceChildren();
});

test("dropping a tile on another tile's centre swaps them", () => {
  const api = dockview();
  drop(a, "right", undefined);
  drop(b, "right", undefined);
  drop(c, "right", undefined);

  drop(a, "center", groupOf(api, "m1:c"));

  expect(tiles(api)).toEqual([["m1:c"], ["m1:b"], ["m1:a"]]);
});

test("every terminal in the workspace gets its own tile", () => {
  const api = dockview();

  sync([a, b, c], () => false);
  expect(tiles(api)).toEqual([["m1:a"], ["m1:b"], ["m1:c"]]);

  sync([a, c], () => false);
  expect(tiles(api)).toEqual([["m1:a"], ["m1:c"]]);
});

test("a saved layout with stacked tiles opens with each tile apart", () => {
  const api = dockview();
  const workspace = { machine: "m1", id: "w1" };
  drop(a, "right", undefined);
  api.addPanel({
    id: "m1:b",
    component: "terminal",
    params: b,
    position: { referenceGroup: groupOf(api, "m1:a"), direction: "within" },
  });
  store.setState({
    workspace: undefined,
    layouts: { [layoutKey(workspace)]: api.toJSON() },
  });

  selectWorkspace(workspace);

  expect(tiles(api)).toEqual([["m1:a"], ["m1:b"]]);
});

test("dropping a tile on the outer edge spans the whole side", () => {
  const api = dockview();
  sync([a, b, c], () => false);

  drop(a, "bottom", undefined);

  expect(groupOf(api, "m1:a").api.width).toBe(900);
  expect(groupOf(api, "m1:b").api.width).toBeLessThan(900);
});

test("a dragged tile drops only where it would move", () => {
  const api = dockview();
  store.setState({ workspace: { machine: "m1", id: "w1" } });
  sync([a, b], () => false);

  startDrag(a);
  expect(canDrop(groupOf(api, "m1:a"), "right")).toBe(false);
  expect(canDrop(groupOf(api, "m1:b"), "center")).toBe(true);
  expect(canDrop(undefined, "bottom")).toBe(true);

  startDrag(c);
  expect(canDrop(groupOf(api, "m1:b"), "center")).toBe(false);
  expect(canDrop(groupOf(api, "m1:b"), "left")).toBe(true);

  startDrag({ machine: "m2", id: "d" });
  expect(canDrop(groupOf(api, "m1:b"), "left")).toBe(false);
  endDrag();
});

test("activating a tile by pointer keeps its content attached", () => {
  const api = dockview();
  sync([a, b], () => false);
  const tile = api.getPanel("m1:a");
  if (!tile) {
    throw new Error("no tile m1:a");
  }

  const changes = new MutationObserver(() => undefined);
  changes.observe(document.body, { childList: true, subtree: true });
  activate(tile.api);
  const removed = changes
    .takeRecords()
    .flatMap(({ removedNodes }) => [...removedNodes]);
  changes.disconnect();

  expect(removed).toEqual([]);
  expect(store.getState().active).toEqual(a);
});

test("opening a workspace focuses only its first terminal", () => {
  dockview();
  const focused: (string | undefined)[] = [];
  const stop = store.subscribe(({ active }) => focused.push(active?.id));

  sync([a, b, c], () => false);
  stop();

  expect(focused.filter((id) => id !== undefined)).toEqual(["a"]);
  expect(store.getState().active).toEqual(a);
});

function addOther(api: DockviewApi, id: string) {
  return api.addPanel({
    id,
    component: "other",
    params: { machine: "m1", id },
    position: { direction: "right" },
  });
}

test("syncing terminals leaves other tiles open", () => {
  const api = dockview();
  sync([a, b], () => false);
  addOther(api, "other:1");

  sync([a], () => false);

  expect(tiles(api)).toEqual([["m1:a"], ["other:1"]]);
});

test("another tile moves by its id and drops only where it exists", () => {
  const api = dockview();
  store.setState({ workspace: { machine: "m1", id: "w1" } });
  sync([a, b], () => false);
  addOther(api, "other:1");

  startDrag({ tile: "other:1" });
  expect(canDrop(groupOf(api, "m1:a"), "center")).toBe(true);
  startDrag({ tile: "other:2" });
  expect(canDrop(groupOf(api, "m1:a"), "left")).toBe(false);
  endDrag();

  drop({ tile: "other:1" }, "center", groupOf(api, "m1:a"));

  expect(tiles(api)).toEqual([["other:1"], ["m1:b"], ["m1:a"]]);
});

test("focusing another tile clears the active terminal", () => {
  const api = dockview();
  sync([a], () => false);
  const other = addOther(api, "other:1");

  activate(other.api);

  expect(store.getState().focused).toBe("other:1");
  expect(store.getState().active).toBeUndefined();
});

test("closing a tile in another workspace removes it from its saved layout", () => {
  const api = dockview();
  const workspace = { machine: "m1", id: "w1" };
  sync([a, b], () => false);
  addOther(api, "other:1");
  store.setState({
    workspace: { machine: "m1", id: "w2" },
    layouts: { [layoutKey(workspace)]: api.toJSON() },
  });
  api.clear();

  closeTile("other:1", workspace);
  selectWorkspace(workspace);

  expect(tiles(api)).toEqual([["m1:a"], ["m1:b"]]);
});
