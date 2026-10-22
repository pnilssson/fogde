import type {
  DockviewApi,
  DockviewGroupPanel,
  DockviewPanelApi,
  IDockviewPanel,
  Position,
} from "dockview-react";
import { positionToDirection } from "dockview-react";

import {
  holdId,
  layoutKey,
  setOpen,
  startId,
  store,
} from "@/features/workspaces/lib/layouts";
import {
  nextWaiting,
  sameView,
  Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import { requestStartFocus } from "@/features/workspaces/lib/start-focus";

let current: DockviewApi | undefined;
let pending: Selection | undefined;
let loading = false;

export function dockview() {
  return current;
}

export function tileId({ machine, id }: Selection) {
  return `${machine}:${id}`;
}

function selectionOf(panel: IDockviewPanel | undefined) {
  if (!panel || panel.id === startId) {
    return undefined;
  }

  return Selection.safeParse(panel.params).data;
}

function focus(panel: IDockviewPanel | undefined) {
  store.setState({
    active: selectionOf(panel),
    startActive: panel?.id === startId,
  });
}

function hideHeader(group: DockviewGroupPanel) {
  group.header.hidden = true;
}

function separate(api: DockviewApi) {
  api.getPanel(holdId)?.api.close();

  for (const group of [...api.groups]) {
    for (const panel of group.panels.slice(1)) {
      panel.api.moveTo({ group, position: "right" });
    }
  }
}

function load(api: DockviewApi) {
  const { workspace, layouts } = store.getState();
  const layout = workspace && layouts[layoutKey(workspace)];

  loading = true;
  try {
    api.clear();
    if (layout) {
      api.fromJSON(layout);
    }
  } catch {
    api.clear();
  } finally {
    loading = false;
  }

  for (const group of api.groups) {
    hideHeader(group);
  }
  separate(api);
}

export function selectWorkspace(next: View) {
  setOpen({ machine: next.machine, id: next.id }, true);
  if (next.checkout) {
    setOpen(next, true);
  }

  if (sameView(next, store.getState().workspace)) {
    return;
  }

  store.setState({ workspace: next });
  if (current) {
    load(current);
  }
}

export function leaveWorkspace(gone: Selection) {
  const { workspace } = store.getState();
  if (gone.machine !== workspace?.machine || gone.id !== workspace.id) {
    return;
  }

  store.setState({ workspace: undefined });
  if (current) {
    load(current);
  }
}

function replace(
  api: DockviewApi,
  selection: Selection,
  group: DockviewGroupPanel,
) {
  const old = group.activePanel;
  api.addPanel({
    id: tileId(selection),
    component: "terminal",
    params: selection,
    position: { referenceGroup: group, direction: "within" },
  });
  old?.api.close();
}

export function place(
  api: DockviewApi,
  selection: Selection,
  position: Position,
  group: DockviewGroupPanel | undefined,
) {
  const direction =
    position === "center" ? "right" : positionToDirection(position);

  api.addPanel({
    id: tileId(selection),
    component: "terminal",
    params: selection,
    position: group ? { referenceGroup: group, direction } : { direction },
  });
}

export function show(selection: Selection) {
  const existing = current?.getPanel(tileId(selection));
  if (existing) {
    existing.focus();
    return;
  }

  pending = selection;
}

export function reveal(workspace: View, terminal: Selection) {
  selectWorkspace(workspace);
  show(terminal);
}

export function revealNext(groups: Parameters<typeof nextWaiting>[0]) {
  const next = nextWaiting(groups, store.getState().active);
  if (next) {
    reveal(next.workspace, next.terminal);
  }
}

function byPlace(groups: DockviewGroupPanel[]) {
  const placed = groups.map((group) => ({
    group,
    rect: group.element.getBoundingClientRect(),
  }));
  placed.sort(
    (a, b) =>
      Math.round(a.rect.left - b.rect.left) ||
      Math.round(a.rect.top - b.rect.top),
  );

  return placed.map(({ group }) => group);
}

export function focusTile(number: number) {
  const group = current && byPlace(current.groups)[number - 1];
  group?.activePanel?.focus();
}

export function activate(panel: DockviewPanelApi) {
  panel.group.api.setActive();
}

export function isShown(selection: Selection) {
  return current?.getPanel(tileId(selection)) !== undefined;
}

export function sync(
  terminals: Selection[],
  keep: (selection: Selection) => boolean,
) {
  if (!current) {
    return;
  }

  const ids = new Set(terminals.map(tileId));
  for (const panel of [...current.panels]) {
    const selection = selectionOf(panel);
    if (selection && !ids.has(panel.id) && !keep(selection)) {
      panel.api.close();
    }
  }

  for (const terminal of terminals) {
    if (!current.getPanel(tileId(terminal))) {
      place(current, terminal, "right", undefined);
    }
  }

  const next = pending;
  if (next && isShown(next)) {
    pending = undefined;
    show(next);
  }
}

export function openStart(folder: string, focus = false) {
  if (!current) {
    return;
  }

  requestStartFocus(focus);

  const existing = current.getPanel(startId);
  if (existing) {
    existing.api.updateParameters({ folder });
    existing.api.setActive();
    return;
  }

  const focused = current.activePanel;
  current.addPanel({
    id: startId,
    component: "start",
    params: { folder },
    position: focused
      ? { referencePanel: focused, direction: "right" }
      : { direction: "right" },
  });
}

export function closeStart() {
  current?.getPanel(startId)?.api.close();
}

export function opened(selection: Selection) {
  const start = current?.getPanel(startId);
  if (!current || !start) {
    show(selection);
    return;
  }

  const existing = current.getPanel(tileId(selection));
  if (existing) {
    existing.api.moveTo({ group: start.group, position: "center" });
    start.api.close();
    return;
  }

  replace(current, selection, start.group);
}

export function restore(api: DockviewApi) {
  load(api);
}

export function attach(api: DockviewApi) {
  current = api;
  const added = api.onDidAddGroup(hideHeader);
  const changed = api.onDidActivePanelChange(({ panel }) => {
    focus(panel);
  });
  const laidOut = api.onDidLayoutChange(() => {
    const { workspace, layouts } = store.getState();
    if (!loading && workspace) {
      store.setState({
        layouts: { ...layouts, [layoutKey(workspace)]: api.toJSON() },
      });
    }
  });

  focus(api.activePanel);

  return () => {
    added.dispose();
    changed.dispose();
    laidOut.dispose();
    current = undefined;
    focus(undefined);
  };
}
