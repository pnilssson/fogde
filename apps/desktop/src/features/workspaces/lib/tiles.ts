import type {
  DockviewApi,
  DockviewGroupPanel,
  DockviewPanelApi,
  IDockviewPanel,
  Position,
} from "dockview-react";
import { positionToDirection } from "dockview-react";

import {
  followedNote,
  followNotes,
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
import { onRename } from "@/lib/note-renames";

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

export function place(
  api: DockviewApi,
  selection: Selection,
  position: Position,
  group: DockviewGroupPanel | undefined,
  inactive = false,
) {
  const direction =
    position === "center" ? "right" : positionToDirection(position);

  api.addPanel({
    id: tileId(selection),
    component: "terminal",
    params: selection,
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

export function revealTile(view: View, id: string) {
  selectWorkspace(view);
  current?.getPanel(id)?.focus();
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
      place(
        current,
        terminal,
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

export function saveLayout() {
  if (current) {
    save(current);
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
  const renamed = onRename((rename) => {
    followNotes(rename);
    for (const panel of api.panels) {
      const params =
        panel.view.contentComponent === "notes"
          ? followedNote(panel.params, rename)
          : undefined;
      if (params) {
        panel.api.updateParameters(params);
      }
    }
    save(api);
  });

  focus(api.activePanel);

  return () => {
    added.dispose();
    changed.dispose();
    laidOut.dispose();
    renamed();
    current = undefined;
    focus(undefined);
  };
}
