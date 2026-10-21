import type {
  DockviewApi,
  DockviewGroupPanel,
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
} from "@/features/terminals/lib/layouts";
import { Selection } from "@/features/terminals/lib/selection";

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

export function selectWorkspace(next: Selection) {
  setOpen(next, true);

  const { workspace } = store.getState();
  if (next.machine === workspace?.machine && next.id === workspace.id) {
    return;
  }

  store.setState({ workspace: next });
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

export function openStart(folder: string) {
  if (!current) {
    return;
  }

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
