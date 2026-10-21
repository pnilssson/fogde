import "dockview-react/dist/styles/dockview.css";

import { PlusCircleIcon } from "@phosphor-icons/react";
import {
  DockviewReact,
  type DockviewApi,
  type DockviewDidDropEvent,
  type DockviewReadyEvent,
  type IDockviewPanelProps,
} from "dockview-react";
import { useContext, useEffect, useState } from "react";

import { StartPage } from "@/features/workspaces/components/start-page";
import { TerminalPane } from "@/features/workspaces/components/terminal-pane";
import { TileBar } from "@/features/workspaces/components/tile-bar";
import {
  isPlacing,
  usePlaceTerminal,
} from "@/features/workspaces/hooks/use-workspaces";
import { Trash } from "@/features/workspaces/components/trash";
import {
  canDrop,
  dragType,
  drop,
  dragged,
  endDrag,
} from "@/features/workspaces/lib/drag";
import {
  holdId,
  startId,
  useStart,
  useTiles,
} from "@/features/workspaces/lib/layouts";
import type { Selection } from "@/features/workspaces/lib/selection";
import {
  startFocused,
  useStartFocus,
} from "@/features/workspaces/lib/start-focus";
import {
  TerminalSlot,
  type RenderTerminal,
} from "@/features/workspaces/lib/slot";
import {
  findTerminal,
  findWorkspace,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import {
  attach,
  closeStart,
  isShown,
  restore,
  sync,
} from "@/features/workspaces/lib/tiles";

const theme = { name: "fogde", className: "dockview-theme-fogde" };

const edges = {
  activationSize: { type: "percentage", value: 15 },
  size: { type: "percentage", value: 15 },
} as const;

function TerminalTile({ api, params }: IDockviewPanelProps<Selection>) {
  const { groups } = useContext(WorkspaceContext);
  const found = findTerminal(groups, params);
  if (!found) {
    return null;
  }

  return (
    <TerminalPane
      panel={api}
      machine={found.machine}
      terminal={found.terminal}
    />
  );
}

function StartTile({ params }: IDockviewPanelProps<{ folder?: string }>) {
  const workspace = useTiles((tiles) => tiles.workspace);
  const active = useTiles((tiles) => tiles.startActive);
  const shared = useStart(workspace) !== undefined;
  const focus = useStartFocus();

  useEffect(() => {
    if (focus) {
      startFocused();
    }
  }, [focus]);

  return (
    <section aria-label="New tile" className="flex h-full min-h-0 flex-col">
      <TileBar
        item={shared ? startId : undefined}
        active={active}
        icon={<PlusCircleIcon className="size-4 shrink-0" />}
        title="New tile"
        onClose={closeStart}
      />
      <div className="flex min-h-0 flex-1 overflow-auto">
        <StartPage folder={params.folder} />
      </div>
    </section>
  );
}

function Watermark() {
  return (
    <div className="flex h-full overflow-auto">
      <StartPage />
    </div>
  );
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

export function TerminalTiles({ terminal }: { terminal: RenderTerminal }) {
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
    <TerminalSlot value={terminal}>
      <div className="relative min-h-0 flex-1">
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
    </TerminalSlot>
  );
}
