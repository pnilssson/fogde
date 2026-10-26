import { NoteItem } from "@/features/workspaces/components/note-item";
import { StartItem } from "@/features/workspaces/components/start-item";
import { TerminalItem } from "@/features/workspaces/components/terminal-item";
import { useFocused } from "@/features/workspaces/hooks/use-focused";
import { useMoveTerminal } from "@/features/workspaces/hooks/use-workspaces";
import {
  startId,
  useNoteTiles,
  useStart,
} from "@/features/workspaces/lib/layouts";
import { changes } from "@/features/workspaces/lib/order";
import {
  inView,
  type Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import { revealTile } from "@/features/workspaces/lib/tiles";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export interface RowsProps {
  machine: Machine;
  workspace: Workspace;
  onSelect: (view: View, terminal?: Selection) => void;
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
  const notes = useNoteTiles(view);
  const focused = useFocused(workspace, view);
  const move = useMoveTerminal();
  const terminals = workspace.terminals.filter((terminal) =>
    inView(view, terminal),
  );
  const ids = terminals.map(({ id }) => id);

  const arrange = (terminal: string, before: string | null) => {
    if (!changes(ids, terminal, before)) {
      return;
    }

    move.mutate({ machine, terminal, before });
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
          onSelect={(selection) => {
            onSelect(view, selection);
          }}
          onArrange={arrange}
        />
      ))}
      {notes.map(({ id, title }) => (
        <NoteItem
          key={id}
          view={view}
          id={id}
          title={title}
          active={focused === id}
          onSelect={() => {
            revealTile(view, id);
          }}
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
