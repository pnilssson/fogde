import {
  GitBranchIcon,
  NotebookIcon,
  PlusCircleIcon,
  TrashIcon,
} from "@phosphor-icons/react";

import { Keys } from "@/components/keys";
import {
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from "@/components/ui/command";
import { TerminalIcon } from "@/features/workspaces/components/terminal-icon";
import {
  useChoices,
  useCreateTerminal,
} from "@/features/workspaces/hooks/use-terminals";
import { useRemoveWorktree } from "@/features/workspaces/hooks/use-worktrees";
import {
  checkoutName,
  checkoutOf,
  mainCheckout,
} from "@/features/workspaces/lib/checkouts";
import { viewFolder, type View } from "@/features/workspaces/lib/selection";
import { openNotes, openStart } from "@/features/workspaces/lib/opening";
import type { Workspace } from "@/generated/protocol";
import { useConnection } from "@/hooks/use-connection";
import { agents } from "@/lib/agents";
import type { Machine } from "@/lib/machines";
import { shortcuts } from "@/lib/shortcuts";

interface Props {
  machine: Machine;
  workspace: Workspace;
  view: View | undefined;
  run: (action: () => void) => void;
  onNewWorktree: () => void;
}

export function PaletteWorkspace({
  machine,
  workspace,
  view,
  run,
  onNewWorktree,
}: Props) {
  const connection = useConnection(machine);
  const choices = useChoices(machine) ?? [null];
  const create = useCreateTerminal();
  const remove = useRemoveWorktree(machine, workspace);
  if (connection.state !== "connected") {
    return null;
  }

  const folder = viewFolder(workspace, view);
  const main = mainCheckout(workspace);
  const checkout = checkoutOf(workspace, folder);

  return (
    <>
      <CommandGroup heading={`New in ${workspace.name}`}>
        <CommandItem
          value="new tile"
          keywords={["new", "open", "start"]}
          onSelect={() => {
            run(() => {
              openStart(folder, true);
            });
          }}
        >
          <PlusCircleIcon />
          New tile
          <CommandShortcut>
            <Keys letter={shortcuts.tile} />
          </CommandShortcut>
        </CommandItem>
        <CommandItem
          value="open notes"
          keywords={["view", "show", "tile"]}
          onSelect={() => {
            run(() => {
              openNotes(machine.id);
            });
          }}
        >
          <NotebookIcon />
          Open notes
        </CommandItem>
        {choices.map((agent) => {
          const name = agent ? agents[agent].name : "terminal";

          return (
            <CommandItem
              key={agent ?? "terminal"}
              value={`new ${name}`}
              keywords={["new", "open", "start"]}
              onSelect={() => {
                create.mutate({
                  machine,
                  workspace: workspace.id,
                  folder,
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
        {main && (
          <CommandItem
            value="new worktree"
            keywords={["new", "create", "branch"]}
            onSelect={() => {
              run(onNewWorktree);
            }}
          >
            <GitBranchIcon />
            New worktree
          </CommandItem>
        )}
      </CommandGroup>
      {checkout && !checkout.main && (
        <CommandGroup heading={checkoutName(checkout)}>
          <CommandItem
            value="remove worktree"
            keywords={["remove", "delete"]}
            onSelect={() => {
              remove.mutate(checkout.path);
              run(() => undefined);
            }}
          >
            <TrashIcon />
            Remove worktree
          </CommandItem>
        </CommandGroup>
      )}
    </>
  );
}
