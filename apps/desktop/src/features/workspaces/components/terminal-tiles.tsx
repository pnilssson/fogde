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

import { NotesPane } from "@/features/workspaces/components/notes-pane";
import { StartPage } from "@/features/workspaces/components/start-page";
import { TerminalPane } from "@/features/workspaces/components/terminal-pane";
import { TileBar } from "@/features/workspaces/components/tile-bar";
import {
  isPlacing,
  useOpenNote,
  usePlaceTerminal,
  useShowNote,
} from "@/features/workspaces/hooks/use-workspaces";
import { Trash } from "@/features/workspaces/components/trash";
import {
  canDrop,
  dragType,
  drop,
  dragged,
  endDrag,
  isTerminal,
} from "@/features/workspaces/lib/drag";
import {
  holdId,
  startId,
  useStart,
  useTiles,
} from "@/features/workspaces/lib/layouts";
import type { Selection } from "@/features/workspaces/lib/selection";
import type { Machine } from "@/lib/machines";
import { draggingNote, noteDragType } from "@/lib/note-drag";
import { pastePath } from "@/lib/paste";
import { report } from "@/lib/report";
import {
  startFocused,
  useStartFocus,
} from "@/features/workspaces/lib/start-focus";
import {
  NotesSlot,
  type RenderNotes,
  TerminalSlot,
  type RenderTerminal,
} from "@/features/workspaces/lib/slot";
import {
  findTerminal,
  findWorkspace,
  inView,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import { canDropNote, dropNote } from "@/features/workspaces/lib/note-drop";
import { closeStart } from "@/features/workspaces/lib/opening";
import {
  activate,
  attach,
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
      workspace={found.workspace}
      terminal={found.terminal}
    />
  );
}

function StartTile({ api, params }: IDockviewPanelProps<{ folder?: string }>) {
  const workspace = useTiles((tiles) => tiles.workspace);
  const active = useTiles((tiles) => tiles.focused === startId);
  const shared = useStart(workspace) !== undefined;
  const focus = useStartFocus();

  useEffect(() => {
    if (focus) {
      startFocused();
    }
  }, [focus]);

  return (
    <section
      aria-label="New tile"
      className="flex h-full min-h-0 flex-col"
      onPointerDown={() => {
        activate(api);
      }}
    >
      <TileBar
        item={shared ? { tile: startId } : undefined}
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
  notes: NotesPane,
  [holdId]: Hold,
};

function accept(event: DockviewReadyEvent) {
  event.api.onUnhandledDragOver((over) => {
    if (!(over.nativeEvent instanceof DragEvent)) {
      return;
    }

    const types = over.nativeEvent.dataTransfer?.types ?? [];
    if (
      (types.includes(dragType) && canDrop(over.group, over.position)) ||
      (types.includes(noteDragType) && canDropNote(over.group, over.position))
    ) {
      over.accept();
    }
  });
}

function paste(machine: Machine, terminal: Selection, path: string) {
  machine.client
    .call("terminals.input", { id: terminal.id, data: pastePath(path) })
    .catch((error: unknown) => {
      report("Could not paste the note path", error);
    });
}

interface Props {
  terminal: RenderTerminal;
  notes: RenderNotes;
}

export function TerminalTiles({ terminal, notes }: Props) {
  const { groups, workspace } = useContext(WorkspaceContext);
  const [api, setApi] = useState<DockviewApi>();
  const place = usePlaceTerminal();
  const open = useOpenNote();
  const show = useShowNote();

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
      found.workspace.tiles
        .filter((tile) => inView(workspace, tile))
        .map(({ kind, id }) => ({ kind, machine: workspace.machine, id })),
      ({ machine, id }) => isPlacing(machine, id),
    );
  }, [api, groups, workspace]);

  const group = groups.find(({ machine }) => machine.id === workspace?.machine);

  const dropped = (event: DockviewDidDropEvent) => {
    if (!(event.nativeEvent instanceof DragEvent)) {
      return;
    }

    const note = draggingNote();
    if (note && event.nativeEvent.dataTransfer?.types.includes(noteDragType)) {
      if (!group || !workspace) {
        return;
      }

      const { machine } = group;
      dropNote(event.position, event.group, {
        paste: (target) => {
          paste(machine, target, note.path);
        },
        show: (tile) => {
          show.mutate({ machine, tile: tile.id, note: note.path });
        },
        open: (spot) => {
          open.mutate({
            machine,
            view: workspace,
            note: note.path,
            spot,
          });
        },
      });
      return;
    }

    const item = dragged(event.nativeEvent);
    endDrag();
    if (!item || !workspace || !group) {
      return;
    }

    if (isTerminal(item) && !isShown(item)) {
      place.mutate({
        machine: group.machine,
        terminal: item.id,
        workspace: workspace.id,
        checkout: workspace.checkout ?? null,
      });
    }
    drop(item, event.position, event.group);
  };

  return (
    <TerminalSlot value={terminal}>
      <NotesSlot value={notes}>
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
      </NotesSlot>
    </TerminalSlot>
  );
}
