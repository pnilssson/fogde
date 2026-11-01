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
  store,
} from "@/features/workspaces/lib/layouts";
import {
  nextWaiting,
  sameView,
  Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import { currentPage, leavePage } from "@/lib/page";

let current: DockviewApi | undefined;
let pending: Selection | undefined;
let loading = false;
let saving = 0;
const settle = 300;

export function dockview() {
  return current;
}

export function tileId({ machine, id }: Selection) {
  return `${machine}:${id}`;
}

function selectionOf(panel: IDockviewPanel | undefined) {
  if (panel?.view.contentComponent !== "terminal") {
    return undefined;
  }

  return Selection.safeParse(panel.params).data;
}

function focus(panel: IDockviewPanel | undefined) {
  store.setState({
    active: selectionOf(panel),
    focused: panel?.id,
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

  for (const group of [...api.groups]) {
    if (group.panels.length === 0) {
      api.removeGroup(group);
    } else {
      hideHeader(group);
    }
  }
  separate(api);
}

export function selectWorkspace(next: View) {
  setOpen({ machine: next.machine, id: next.id }, true);

  if (sameView(next, store.getState().workspace)) {
    return;
  }

  flush();
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

  flush();
  store.setState({ workspace: undefined });
  if (current) {
    load(current);
  }
}

function tilePanel({ machine, id }: Selection) {
  const selection = { machine, id };

  return {
    id: tileId(selection),
    component: "terminal",
    params: selection,
  };
}

export function place(
  api: DockviewApi,
  tile: Selection,
  position: Position,
  group: DockviewGroupPanel | undefined,
  inactive = false,
) {
  const direction =
    position === "center" ? "right" : positionToDirection(position);

  api.addPanel({
    ...tilePanel(tile),
    position: group ? { referenceGroup: group, direction } : { direction },
    inactive,
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

export function showTiles() {
  leavePage(() => {
    const { active } = store.getState();
    if (active) {
      show(active);
    }
  });
}

export function reveal(workspace: View, terminal: Selection) {
  selectWorkspace(workspace);
  leavePage(() => {
    show(terminal);
  });
}

export function revealNext(groups: Parameters<typeof nextWaiting>[0]) {
  const next = nextWaiting(groups, store.getState().active);
  if (next) {
    reveal(next.workspace, next.terminal);
  }
}

export function toggleZoom(panel = current?.activePanel?.api) {
  if (!panel) {
    return;
  }

  if (panel.isMaximized()) {
    panel.exitMaximized();
    return;
  }

  panel.maximize();
}

export function activate(panel: DockviewPanelApi) {
  panel.group.api.setActive();
}

export function hasTile(selection: Selection) {
  return current?.getPanel(tileId(selection)) !== undefined;
}

export function isShown(selection: Selection) {
  const panel = current?.getPanel(tileId(selection));
  if (!current || !panel || currentPage()) {
    return false;
  }

  return !current.hasMaximizedGroup() || panel.api.isMaximized();
}

export function sync(
  tiles: Selection[],
  keep: (selection: Selection) => boolean,
) {
  if (!current) {
    return;
  }

  const zoomed = current.panels.find((panel) => panel.api.isMaximized());
  const ids = new Set(tiles.map(tileId));
  for (const panel of [...current.panels]) {
    const selection = selectionOf(panel);
    if (!selection || ids.has(panel.id)) {
      continue;
    }

    if (!keep(selection)) {
      panel.api.close();
    }
  }

  for (const tile of tiles) {
    if (!current.getPanel(tileId(tile))) {
      place(
        current,
        tile,
        "right",
        undefined,
        current.activePanel !== undefined,
      );
    }
  }

  if (zoomed && current.getPanel(zoomed.id) && !zoomed.api.isMaximized()) {
    zoomed.api.maximize();
  }

  const next = pending;
  if (next && hasTile(next)) {
    pending = undefined;
    show(next);
  }
}

export function restore(api: DockviewApi) {
  load(api);
}

function flush() {
  window.clearTimeout(saving);
  if (current) {
    save(current);
  }
}

function save(api: DockviewApi) {
  const { workspace, layouts } = store.getState();
  if (!loading && workspace) {
    store.setState({
      layouts: { ...layouts, [layoutKey(workspace)]: api.toJSON() },
    });
  }
}

export function attach(api: DockviewApi) {
  current = api;
  const added = api.onDidAddGroup(hideHeader);
  const changed = api.onDidActivePanelChange(({ panel }) => {
    focus(panel);
  });
  const laidOut = api.onDidLayoutChange(() => {
    window.clearTimeout(saving);
    saving = window.setTimeout(() => {
      save(api);
    }, settle);
  });

  focus(api.activePanel);

  return () => {
    added.dispose();
    changed.dispose();
    laidOut.dispose();
    flush();
    current = undefined;
    focus(undefined);
  };
}
