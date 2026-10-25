import { NoteItem } from "@/features/workspaces/components/note-item";
import { StartItem } from "@/features/workspaces/components/start-item";
import { TerminalItem } from "@/features/workspaces/components/terminal-item";
import { useFocused } from "@/features/workspaces/hooks/use-focused";
import {
  startId,
  useNoteTiles,
  useStart,
} from "@/features/workspaces/lib/layouts";
import {
  inView,
  type Selection,
  type View,
} from "@/features/workspaces/lib/selection";
import { byAttention } from "@/features/workspaces/lib/status";
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

  return (
    <>
      {byAttention(
        workspace.terminals.filter((terminal) => inView(view, terminal)),
      ).map((terminal) => (
        <TerminalItem
          key={terminal.id}
          machine={machine}
          terminal={terminal}
          active={focused === terminal.id}
          onSelect={(selection) => {
            onSelect(view, selection);
          }}
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
