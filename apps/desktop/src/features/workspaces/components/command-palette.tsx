import { BellIcon, FolderIcon, PlusCircleIcon } from "@phosphor-icons/react";
import { useContext, useRef } from "react";

import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from "@/components/ui/command";
import { Keys } from "@/components/keys";
import {
  setPaletteOpen,
  usePaletteOpen,
} from "@/features/workspaces/lib/palette";
import {
  useChoices,
  useCreateTerminal,
} from "@/features/workspaces/hooks/use-terminals";
import { PaletteTiles } from "@/features/workspaces/components/palette-tiles";
import { TerminalIcon } from "@/features/workspaces/components/terminal-icon";
import {
  findWorkspace,
  waiting,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import {
  dockview,
  openStart,
  reveal,
  revealNext,
  selectWorkspace,
} from "@/features/workspaces/lib/tiles";
import { useConnection } from "@/hooks/use-connection";
import type { Workspace } from "@/generated/protocol";
import { agentName } from "@/lib/agents";
import type { Machine } from "@/lib/machines";
import { shortcuts } from "@/lib/shortcuts";

type Run = (action: () => void) => void;

export function CommandPalette() {
  const open = usePaletteOpen();
  const pending = useRef<(() => void) | undefined>(undefined);

  const run: Run = (action) => {
    pending.current = action;
    setPaletteOpen(false);
  };

  return (
    <CommandDialog
      open={open}
      onOpenChange={setPaletteOpen}
      onOpenChangeComplete={(next) => {
        if (next) {
          return;
        }

        const action = pending.current;
        pending.current = undefined;
        action?.();
      }}
      finalFocus={() => pending.current === undefined}
      title="Commands"
      description="Go to a terminal or workspace, or run a command."
    >
      <Command loop>
        <CommandInput placeholder="Go to or run…" />
        <CommandList>
          <CommandEmpty>Nothing matches.</CommandEmpty>
          <Commands run={run} />
        </CommandList>
      </Command>
    </CommandDialog>
  );
}

function Commands({ run }: { run: Run }) {
  const { groups, workspace } = useContext(WorkspaceContext);
  const found = findWorkspace(groups, workspace);
  const needs = waiting(groups);
  const shown = new Set(needs.map(({ terminal }) => key(terminal)));
  const several = groups.length > 1;
  const tiles = dockview()?.groups.length ?? 0;

  const place = (machine: Machine, workspace: Workspace) =>
    several ? `${workspace.name} · ${machine.name}` : workspace.name;

  return (
    <>
      {needs.length > 0 && (
        <CommandGroup heading="Needs you">
          <CommandItem
            value="next agent that needs you"
            onSelect={() => {
              run(() => {
                revealNext(groups);
              });
            }}
          >
            <BellIcon />
            Go to the next agent that needs you
            <CommandShortcut>
              <Keys letter={shortcuts.attention} />
            </CommandShortcut>
          </CommandItem>
          {needs.map((item) => (
            <CommandItem
              key={key(item.terminal)}
              value={`waiting ${key(item.terminal)}`}
              keywords={[item.name, item.place]}
              onSelect={() => {
                run(() => {
                  reveal(item.workspace, item.terminal);
                });
              }}
            >
              <TerminalIcon agent={item.agent} />
              <Label name={item.name} place={item.place} />
            </CommandItem>
          ))}
        </CommandGroup>
      )}
      {found && (
        <NewTerminals
          machine={found.group.machine}
          workspace={found.workspace}
          run={run}
        />
      )}
      <CommandGroup heading="Terminals">
        {groups.flatMap(({ machine, workspaces }) =>
          (workspaces.data ?? []).flatMap((current) =>
            current.terminals
              .filter(({ id }) => !shown.has(key({ machine: machine.id, id })))
              .map((terminal) => {
                const selection = { machine: machine.id, id: terminal.id };

                return (
                  <CommandItem
                    key={key(selection)}
                    value={`terminal ${key(selection)}`}
                    keywords={[terminal.name, current.name]}
                    onSelect={() => {
                      run(() => {
                        reveal(
                          { machine: machine.id, id: current.id },
                          selection,
                        );
                      });
                    }}
                  >
                    <TerminalIcon agent={terminal.agent} />
                    <Label
                      name={terminal.name}
                      place={place(machine, current)}
                    />
                  </CommandItem>
                );
              }),
          ),
        )}
      </CommandGroup>
      <CommandGroup heading="Workspaces">
        {groups.flatMap(({ machine, workspaces }) =>
          (workspaces.data ?? []).map((current) => {
            const selection = { machine: machine.id, id: current.id };

            return (
              <CommandItem
                key={key(selection)}
                value={`workspace ${key(selection)}`}
                keywords={[current.name, machine.name]}
                onSelect={() => {
                  run(() => {
                    selectWorkspace(selection);
                  });
                }}
              >
                <FolderIcon />
                <Label
                  name={current.name}
                  place={several ? machine.name : undefined}
                />
              </CommandItem>
            );
          }),
        )}
      </CommandGroup>
      {tiles > 0 && <PaletteTiles tiles={tiles} run={run} />}
    </>
  );
}

interface NewProps {
  machine: Machine;
  workspace: Workspace;
  run: Run;
}

function NewTerminals({ machine, workspace, run }: NewProps) {
  const connection = useConnection(machine);
  const choices = useChoices(machine);
  const create = useCreateTerminal();
  if (connection.state !== "connected") {
    return null;
  }

  return (
    <CommandGroup heading={`New in ${workspace.name}`}>
      <CommandItem
        value="new tile"
        keywords={["new", "open", "start"]}
        onSelect={() => {
          run(() => {
            openStart(workspace.folder, true);
          });
        }}
      >
        <PlusCircleIcon />
        New tile
        <CommandShortcut>
          <Keys letter={shortcuts.tile} />
        </CommandShortcut>
      </CommandItem>
      {choices.map((agent) => {
        const name = agent ? agentName[agent] : "terminal";

        return (
          <CommandItem
            key={agent ?? "terminal"}
            value={`new ${name}`}
            keywords={["new", "open", "start"]}
            onSelect={() => {
              create.mutate({
                machine,
                workspace: workspace.id,
                folder: workspace.folder,
                agent,
              });
              run(() => undefined);
            }}
          >
            <TerminalIcon agent={agent} />
            New {name}
          </CommandItem>
        );
      })}
    </CommandGroup>
  );
}

function Label({ name, place }: { name: string; place?: string | undefined }) {
  return (
    <span className="flex min-w-0 gap-2">
      <span className="truncate">{name}</span>
      {place && <span className="truncate text-muted-foreground">{place}</span>}
    </span>
  );
}

function key({ machine, id }: { machine: string; id: string }) {
  return `${machine}/${id}`;
}
