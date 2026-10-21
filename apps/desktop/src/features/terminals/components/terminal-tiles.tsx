import "dockview-react/dist/styles/dockview.css";

import {
  DockviewReact,
  type DockviewApi,
  type DockviewDidDropEvent,
  type DockviewReadyEvent,
  type IDockviewPanelProps,
} from "dockview-react";
import { type ReactNode, useContext, useEffect, useState } from "react";

import { StartPage } from "@/features/terminals/components/start-page";
import { TerminalPane } from "@/features/terminals/components/terminal-pane";
import { TileGrip } from "@/features/terminals/components/tile-grip";
import {
  isPlacing,
  usePlaceTerminal,
} from "@/features/terminals/hooks/use-workspaces";
import { Trash } from "@/features/terminals/components/trash";
import {
  canDrop,
  dragType,
  drop,
  dragged,
  endDrag,
} from "@/features/terminals/lib/drag";
import {
  holdId,
  startId,
  useStart,
  useTiles,
} from "@/features/terminals/lib/layouts";
import type { Selection } from "@/features/terminals/lib/selection";
import {
  findTerminal,
  findWorkspace,
  WorkspaceContext,
} from "@/features/terminals/lib/selection";
import { attach, isShown, restore, sync } from "@/features/terminals/lib/tiles";

const theme = { name: "fogde", className: "dockview-theme-fogde" };

const edges = {
  activationSize: { type: "percentage", value: 15 },
  size: { type: "percentage", value: 15 },
} as const;

function Tile({ children }: { children: ReactNode }) {
  return <div className="h-full p-1">{children}</div>;
}

function TerminalTile({ api, params }: IDockviewPanelProps<Selection>) {
  const { groups } = useContext(WorkspaceContext);
  const found = findTerminal(groups, params);
  if (!found) {
    return null;
  }

  return (
    <Tile>
      <TerminalPane
        panel={api}
        machine={found.machine}
        terminal={found.terminal}
      />
    </Tile>
  );
}

interface StartProps {
  folder?: string | undefined;
  movable?: boolean;
}

function StartSurface({ folder, movable = false }: StartProps) {
  return (
    <Tile>
      <section
        aria-label="Start"
        className="group/tile relative flex h-full min-h-0 flex-col overflow-hidden rounded-xl bg-surface-3 shadow-surface-3"
      >
        {movable && <TileGrip item={startId} />}
        <div className="flex min-h-0 flex-1 overflow-auto">
          <StartPage folder={folder} />
        </div>
      </section>
    </Tile>
  );
}

function StartTile({ params }: IDockviewPanelProps<{ folder?: string }>) {
  const workspace = useTiles((tiles) => tiles.workspace);
  const shared = useStart(workspace) !== undefined;

  return <StartSurface folder={params.folder} movable={shared} />;
}

function Watermark() {
  return <StartSurface />;
}

function Hold() {
  return null;
}

const components = {
  terminal: TerminalTile,
  start: StartTile,
  [holdId]: Hold,
};

function accept(event: DockviewReadyEvent) {
  event.api.onUnhandledDragOver((over) => {
    if (
      over.nativeEvent instanceof DragEvent &&
      over.nativeEvent.dataTransfer?.types.includes(dragType) &&
      canDrop(over.group, over.position)
    ) {
      over.accept();
    }
  });
}

export function TerminalTiles() {
  const { groups, workspace } = useContext(WorkspaceContext);
  const [api, setApi] = useState<DockviewApi>();
  const place = usePlaceTerminal();

  useEffect(() => (api ? attach(api) : undefined), [api]);

  useEffect(() => {
    if (!api || !workspace) {
      return;
    }

    const found = findWorkspace(groups, workspace);
    if (!found) {
      return;
    }

    sync(
      found.workspace.terminals.map(({ id }) => ({
        machine: workspace.machine,
        id,
      })),
      ({ machine, id }) => isPlacing(machine, id),
    );
  }, [api, groups, workspace]);

  const group = groups.find(({ machine }) => machine.id === workspace?.machine);

  const dropped = (event: DockviewDidDropEvent) => {
    if (!(event.nativeEvent instanceof DragEvent)) {
      return;
    }

    const item = dragged(event.nativeEvent);
    endDrag();
    if (!item || !workspace || !group) {
      return;
    }

    if (item !== startId && !isShown(item)) {
      place.mutate({
        machine: group.machine,
        terminal: item.id,
        workspace: workspace.id,
      });
    }
    drop(item, event.position, event.group);
  };

  return (
    <div className="relative min-h-0 flex-1 p-1">
      <DockviewReact
        className="isolate"
        theme={theme}
        components={components}
        watermarkComponent={Watermark}
        disableFloatingGroups
        dndEdges={edges}
        onReady={(event) => {
          accept(event);
          restore(event.api);
          setApi(event.api);
        }}
        onDidDrop={dropped}
      />
      {group && <Trash machine={group.machine} />}
    </div>
  );
}
