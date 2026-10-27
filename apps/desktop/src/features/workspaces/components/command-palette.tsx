import { BellIcon, FolderIcon, GearSixIcon } from "@phosphor-icons/react";
import { useContext, useRef, useState } from "react";

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
import { NewWorktree } from "@/features/workspaces/components/new-worktree";
import { PalettePages } from "@/features/workspaces/components/palette-pages";
import { PaletteTiles } from "@/features/workspaces/components/palette-tiles";
import { PaletteWorkspace } from "@/features/workspaces/components/palette-workspace";
import { RemoveWorktree } from "@/features/workspaces/components/remove-worktree";
import { TerminalIcon } from "@/features/workspaces/components/terminal-icon";
import {
  findWorkspace,
  viewOf,
  waiting,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import {
  dockview,
  reveal,
  revealNext,
  selectWorkspace,
} from "@/features/workspaces/lib/tiles";
import type { RenderPaletteNotes } from "@/features/workspaces/lib/slot";
import type { Checkout, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { closePage, openNote } from "@/lib/page";
import { setSettingsOpen } from "@/lib/settings-open";
import { shortcuts } from "@/lib/shortcuts";

type Run = (action: () => void) => void;

interface CommandsProps {
  run: Run;
  notes: RenderPaletteNotes;
  onNewWorktree: () => void;
  onRemoveWorktree: (checkout: Checkout) => void;
}

export function CommandPalette({ notes }: { notes: RenderPaletteNotes }) {
  const { groups, workspace } = useContext(WorkspaceContext);
  const found = findWorkspace(groups, workspace);
  const open = usePaletteOpen();
  const pending = useRef<(() => void) | undefined>(undefined);
  const [naming, setNaming] = useState(false);
  const [removing, setRemoving] = useState<Checkout>();
  const [confirming, setConfirming] = useState(false);

  const run: Run = (action) => {
    pending.current = action;
    setPaletteOpen(false);
  };

  return (
    <>
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
            <Commands
              run={run}
              notes={notes}
              onNewWorktree={() => {
                setNaming(true);
              }}
              onRemoveWorktree={(checkout) => {
                setRemoving(checkout);
                setConfirming(true);
              }}
            />
          </CommandList>
        </Command>
      </CommandDialog>
      {found && (
        <NewWorktree
          machine={found.group.machine}
          workspace={found.workspace}
          open={naming}
          onOpenChange={setNaming}
        />
      )}
      {found && removing && (
        <RemoveWorktree
          machine={found.group.machine}
          workspace={found.workspace}
          checkout={removing}
          open={confirming}
          onOpenChange={setConfirming}
        />
      )}
    </>
  );
}

function Commands({
  run,
  notes,
  onNewWorktree,
  onRemoveWorktree,
}: CommandsProps) {
  const { groups, workspace } = useContext(WorkspaceContext);
  const found = findWorkspace(groups, workspace);
  const notesMachine = found?.group.machine ?? groups[0]?.machine;
  const needs = waiting(groups);
  const shown = new Set(needs.map(({ terminal }) => key(terminal)));
  const several = groups.length > 1;
  const tiles = dockview()?.groups.length ?? 0;

  const place = (machine: Machine, workspace: Workspace) =>
    several ? `${workspace.name} on ${machine.name}` : workspace.name;

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
      {found && !found.workspace.missing && (
        <PaletteWorkspace
          machine={found.group.machine}
          workspace={found.workspace}
          view={workspace}
          run={run}
          onNewWorktree={onNewWorktree}
          onRemoveWorktree={onRemoveWorktree}
        />
      )}
      {notesMachine &&
        notes({
          machine: notesMachine,
          onOpen: (note) => {
            run(() => {
              openNote(notesMachine.id, note);
            });
          },
        })}
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
                          viewOf(machine.id, current, terminal),
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
                    closePage();
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
      <PalettePages machine={workspace?.machine} run={run} />
      <CommandGroup heading="App">
        <CommandItem
          value="Settings"
          onSelect={() => {
            run(() => {
              setSettingsOpen(true);
            });
          }}
        >
          <GearSixIcon />
          Settings
          <CommandShortcut>
            <Keys letter={shortcuts.settings} />
          </CommandShortcut>
        </CommandItem>
      </CommandGroup>
    </>
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
