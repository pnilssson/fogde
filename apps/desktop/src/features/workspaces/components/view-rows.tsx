import { StartItem } from "@/features/workspaces/components/start-item";
import { TerminalItem } from "@/features/workspaces/components/terminal-item";
import { useFocused } from "@/features/workspaces/hooks/use-focused";
import { useMoveTile } from "@/features/workspaces/hooks/use-workspaces";
import { startId, useStart } from "@/features/workspaces/lib/layouts";
import { changes } from "@/features/workspaces/lib/order";
import {
  inView,
  type Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export interface RowsProps {
  machine: Machine;
  workspace: Workspace;
  onSelect: (view: View, tile?: Selection) => void;
  onStart: (view: View, folder: string) => void;
}

interface Props extends RowsProps {
  view: View;
}

export function ViewRows({
  machine,
  workspace,
  view,
  onSelect,
  onStart,
}: Props) {
  const start = useStart(view);
  const focused = useFocused(workspace, view);
  const move = useMoveTile();
  const terminals = workspace.terminals.filter((terminal) =>
    inView(view, terminal),
  );
  const ids = terminals.map(({ id }) => id);

  const arrange = (tile: string, before: string | null) => {
    if (!changes(ids, tile, before)) {
      return;
    }

    move.mutate({ machine, tile, before });
  };

  return (
    <>
      {terminals.map((terminal) => (
        <TerminalItem
          key={terminal.id}
          machine={machine}
          terminal={terminal}
          siblings={ids}
          active={focused === terminal.id}
          onSelect={() => {
            onSelect(view, { machine: machine.id, id: terminal.id });
          }}
          onArrange={arrange}
        />
      ))}
      {start !== undefined && (
        <StartItem
          view={view}
          active={focused === startId}
          onSelect={() => {
            onStart(view, start);
          }}
        />
      )}
    </>
  );
}
