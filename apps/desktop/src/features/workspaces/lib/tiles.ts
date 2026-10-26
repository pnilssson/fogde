import type {
  DockviewApi,
  DockviewGroupPanel,
  DockviewPanelApi,
  IDockviewPanel,
  Position,
} from "dockview-react";
import { positionToDirection } from "dockview-react";

import type { Listed } from "@/features/workspaces/lib/drag";
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

let current: DockviewApi | undefined;
let pending: Selection | undefined;
let loading = false;

export function dockview() {
  return current;
}

export function tileId({ machine, id }: Selection) {
  return `${machine}:${id}`;
}

const components = { terminal: "terminal", note: "notes" } as const;

function listedOf(panel: IDockviewPanel) {
  const kind = Object.entries(components).find(
    ([, component]) => component === panel.view.contentComponent,
  )?.[0];

  return kind === undefined
    ? undefined
    : { kind, selection: Selection.safeParse(panel.params).data };
}

function selectionOf(panel: IDockviewPanel | undefined) {
  if (panel?.view.contentComponent !== components.terminal) {
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

export function tilePanel({ kind, machine, id }: Listed) {
  const selection = { machine, id };

  return {
    id: tileId(selection),
    component: components[kind],
    params: selection,
  };
}

export function place(
  api: DockviewApi,
  tile: Listed,
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

export function activate(panel: DockviewPanelApi) {
  panel.group.api.setActive();
}

export function isShown(selection: Selection) {
  return current?.getPanel(tileId(selection)) !== undefined;
}

export function sync(tiles: Listed[], keep: (selection: Selection) => boolean) {
  if (!current) {
    return;
  }

  const ids = new Set(tiles.map(tileId));
  for (const panel of [...current.panels]) {
    const listed = listedOf(panel);
    if (!listed || ids.has(panel.id)) {
      continue;
    }

    const kept =
      listed.kind === "terminal" &&
      listed.selection !== undefined &&
      keep(listed.selection);
    if (!kept) {
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

  const next = pending;
  if (next && isShown(next)) {
    pending = undefined;
    show(next);
  }
}

export function restore(api: DockviewApi) {
  load(api);
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
    save(api);
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
