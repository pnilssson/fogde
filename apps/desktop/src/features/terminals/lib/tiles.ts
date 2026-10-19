import type {
  DockviewApi,
  DockviewGroupPanel,
  IDockviewPanel,
  Position,
} from "dockview-react";
import { positionToDirection } from "dockview-react";

import { layoutKey, startId, store } from "@/features/terminals/lib/layouts";
import { Selection } from "@/features/terminals/lib/selection";

export const dragType = "application/x-fogde-terminal";

let current: DockviewApi | undefined;
let pending: Selection | undefined;
let dragging: Selection | undefined;
let loading = false;

function tileId({ machine, id }: Selection) {
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
}

export function selectWorkspace(next: Selection) {
  const { workspace } = store.getState();
  if (next.machine === workspace?.machine && next.id === workspace.id) {
    return;
  }

  store.setState({ workspace: next });
  if (current) {
    load(current);
  }
}

export function dragged(event: DragEvent) {
  try {
    const data = event.dataTransfer?.getData(dragType);

    return data ? Selection.safeParse(JSON.parse(data)).data : undefined;
  } catch {
    return undefined;
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

export function show(selection: Selection) {
  if (!current) {
    pending = selection;
    return;
  }

  const existing = current.getPanel(tileId(selection));
  if (existing) {
    existing.focus();
    return;
  }

  const focused = current.activePanel;
  if (focused) {
    replace(current, selection, focused.group);
    return;
  }

  current.addPanel({
    id: tileId(selection),
    component: "terminal",
    params: selection,
  });
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
    position: focused && { referencePanel: focused, direction: "right" },
  });
}

export function opened(selection: Selection) {
  const start = current?.getPanel(startId);
  if (current && start) {
    replace(current, selection, start.group);
    return;
  }

  show(selection);
}

export function startDrag(selection: Selection) {
  dragging = selection;
}

export function endDrag() {
  dragging = undefined;
}

export function canDrop(group: DockviewGroupPanel | undefined) {
  if (!dragging || dragging.machine !== store.getState().workspace?.machine) {
    return false;
  }

  const existing = current?.getPanel(tileId(dragging));

  return !existing || (group !== undefined && group !== existing.group);
}

export function drop(
  selection: Selection,
  position: Position,
  group: DockviewGroupPanel | undefined,
) {
  if (!current) {
    return;
  }

  const existing = current.getPanel(tileId(selection));
  if (existing) {
    if (group && group !== existing.group) {
      existing.api.moveTo({ group, position });
    }

    return;
  }

  if (group && position === "center") {
    replace(current, selection, group);
    return;
  }

  current.addPanel({
    id: tileId(selection),
    component: "terminal",
    params: selection,
    position: group
      ? { referenceGroup: group, direction: positionToDirection(position) }
      : { direction: positionToDirection(position) },
  });
}

export function prune(keep: (selection: Selection) => boolean) {
  for (const panel of current?.panels ?? []) {
    const selection = selectionOf(panel);
    if (selection && !keep(selection)) {
      panel.api.close();
    }
  }
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
  if (pending) {
    show(pending);
    pending = undefined;
  }

  return () => {
    added.dispose();
    changed.dispose();
    laidOut.dispose();
    current = undefined;
    focus(undefined);
  };
}
