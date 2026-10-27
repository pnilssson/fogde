import {
  GitBranchIcon,
  PlusCircleIcon,
  TrashIcon,
} from "@phosphor-icons/react";

import { Keys } from "@/components/keys";
import {
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from "@/components/ui/command";
import { Spinner } from "@/components/ui/spinner";
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
  working,
} from "@/features/workspaces/lib/checkouts";
import { viewFolder, type View } from "@/features/workspaces/lib/selection";
import { openStart } from "@/features/workspaces/lib/opening";
import type { Checkout, Workspace } from "@/generated/protocol";
import { useConnection } from "@/hooks/use-connection";
import { agents, allAgents } from "@/lib/agents";
import type { Machine } from "@/lib/machines";
import { closePage, leavePage } from "@/lib/page";
import { shortcuts } from "@/lib/shortcuts";

const searched = allAgents.flatMap((agent) => [
  agents[agent].name,
  agents[agent].short,
]);

interface Props {
  machine: Machine;
  workspace: Workspace;
  view: View | undefined;
  run: (action: () => void) => void;
  onNewWorktree: () => void;
  onRemoveWorktree: (checkout: Checkout) => void;
}

export function PaletteWorkspace({
  machine,
  workspace,
  view,
  run,
  onNewWorktree,
  onRemoveWorktree,
}: Props) {
  const connection = useConnection(machine);
  const choices = useChoices(machine);
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
              leavePage(() => {
                openStart(folder, true);
              });
            });
          }}
        >
          <PlusCircleIcon />
          New tile
          <CommandShortcut>
            <Keys letter={shortcuts.tile} />
          </CommandShortcut>
        </CommandItem>
        {(choices ?? [null]).map((agent) => {
          const name = agent ? agents[agent].name : "terminal";

          return (
            <CommandItem
              key={agent ?? "terminal"}
              value={`new ${name}`}
              keywords={["new", "open", "start"]}
              onSelect={() => {
                closePage();
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
        {!choices && (
          <CommandItem
            value="loading agents"
            keywords={["new", "open", "start", ...searched]}
            disabled
          >
            <Spinner aria-hidden />
            Loading agents…
          </CommandItem>
        )}
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
              if (working(workspace, checkout.path).length > 0) {
                run(() => {
                  onRemoveWorktree(checkout);
                });
                return;
              }

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
