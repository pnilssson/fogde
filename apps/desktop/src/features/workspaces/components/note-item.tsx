import { NotebookIcon } from "@phosphor-icons/react";

import { TileItem } from "@/features/workspaces/components/tile-item";
import { useCloseNote } from "@/features/workspaces/hooks/use-workspaces";
import { type NoteTile, noteTitle } from "@/features/workspaces/lib/tile-kinds";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  note: NoteTile;
  siblings: string[];
  active: boolean;
  onSelect: () => void;
  onArrange: (tile: string, before: string | null) => void;
}

export function NoteItem({
  machine,
  note,
  siblings,
  active,
  onSelect,
  onArrange,
}: Props) {
  const close = useCloseNote();

  return (
    <TileItem
      tile={{ kind: "note", machine: machine.id, id: note.id }}
      siblings={siblings}
      active={active}
      icon={<NotebookIcon />}
      label={noteTitle(note.note)}
      closable
      onSelect={onSelect}
      onArrange={onArrange}
      onClose={() => {
        close.mutate({ machine, tile: note.id });
      }}
    />
  );
}
