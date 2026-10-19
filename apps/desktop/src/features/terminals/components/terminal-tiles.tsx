import "dockview-react/dist/styles/dockview.css";

import { XIcon } from "@phosphor-icons/react";
import {
  DockviewReact,
  type DockviewApi,
  type DockviewDidDropEvent,
  type DockviewReadyEvent,
  type IDockviewPanelProps,
} from "dockview-react";
import { type ReactNode, useContext, useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { StartPage } from "@/features/terminals/components/start-page";
import { TerminalPane } from "@/features/terminals/components/terminal-pane";
import {
  isPlacing,
  usePlaceTerminal,
} from "@/features/terminals/hooks/use-workspaces";
import { useStart, useTiles } from "@/features/terminals/lib/layouts";
import type { Selection } from "@/features/terminals/lib/selection";
import {
  findTerminal,
  WorkspaceContext,
} from "@/features/terminals/lib/selection";
import {
  attach,
  canDrop,
  dragType,
  drop,
  dragged,
  prune,
  restore,
} from "@/features/terminals/lib/tiles";

const theme = { name: "fogde", className: "dockview-theme-fogde" };

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
  onClose?: (() => void) | undefined;
}

function StartSurface({ folder, onClose }: StartProps) {
  return (
    <Tile>
      <section
        aria-label="Start"
        className="flex h-full min-h-0 flex-col overflow-hidden rounded-xl bg-surface-3 shadow-surface-3"
      >
        <header className="flex h-8 shrink-0 items-center justify-end px-2">
          {onClose && (
            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    variant="ghost"
                    size="sm"
                    aria-label="Close new tile"
                    onClick={onClose}
                  />
                }
              >
                <XIcon />
              </TooltipTrigger>
              <TooltipContent>Close</TooltipContent>
            </Tooltip>
          )}
        </header>
        <div className="flex min-h-0 flex-1 overflow-auto">
          <StartPage folder={folder} />
        </div>
      </section>
    </Tile>
  );
}

function StartTile({ api, params }: IDockviewPanelProps<{ folder?: string }>) {
  const workspace = useTiles((tiles) => tiles.workspace);
  const shared = useStart(workspace) !== undefined;

  return (
    <StartSurface
      folder={params.folder}
      onClose={
        shared
          ? () => {
              api.close();
            }
          : undefined
      }
    />
  );
}

function Watermark() {
  return <StartSurface />;
}

const components = { terminal: TerminalTile, start: StartTile };

function accept(event: DockviewReadyEvent) {
  event.api.onUnhandledDragOver((over) => {
    if (
      over.nativeEvent instanceof DragEvent &&
      over.nativeEvent.dataTransfer?.types.includes(dragType) &&
      canDrop(over.group)
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

    prune(({ machine, id }) => {
      const workspaces = groups.find((group) => group.machine.id === machine)
        ?.workspaces.data;
      if (!workspaces) {
        return true;
      }

      const owner = workspaces.find(({ terminals }) =>
        terminals.some((terminal) => terminal.id === id),
      );

      return (
        machine === workspace.machine &&
        (owner?.id === workspace.id || isPlacing(machine, id))
      );
    });
  }, [api, groups, workspace]);

  const dropped = (event: DockviewDidDropEvent) => {
    if (!(event.nativeEvent instanceof DragEvent)) {
      return;
    }

    const selection = dragged(event.nativeEvent);
    const group = groups.find(
      ({ machine }) => machine.id === workspace?.machine,
    );
    if (!selection || !workspace || !group) {
      return;
    }

    place.mutate({
      machine: group.machine,
      terminal: selection.id,
      workspace: workspace.id,
    });
    drop(selection, event.position, event.group);
  };

  return (
    <div className="min-h-0 flex-1 p-1">
      <DockviewReact
        theme={theme}
        components={components}
        watermarkComponent={Watermark}
        disableFloatingGroups
        onReady={(event) => {
          accept(event);
          restore(event.api);
          setApi(event.api);
        }}
        onDidDrop={dropped}
      />
    </div>
  );
}
