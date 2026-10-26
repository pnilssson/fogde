import { NotebookIcon } from "@phosphor-icons/react";
import type { IDockviewPanelProps } from "dockview-react";
import { useContext } from "react";

import { TileBar } from "@/features/workspaces/components/tile-bar";
import {
  useCloseTile,
  useShowNote,
} from "@/features/workspaces/hooks/use-workspaces";
import { useTiles } from "@/features/workspaces/lib/layouts";
import {
  findNote,
  type Selection,
  WorkspaceContext,
} from "@/features/workspaces/lib/selection";
import { NotesSlot } from "@/features/workspaces/lib/slot";
import { noteTitle } from "@/features/workspaces/lib/tile-kinds";
import { activate } from "@/features/workspaces/lib/tiles";

export function NotesPane({ api, params }: IDockviewPanelProps<Selection>) {
  const render = useContext(NotesSlot);
  const { groups } = useContext(WorkspaceContext);
  const active = useTiles((tiles) => tiles.focused === api.id);
  const show = useShowNote();
  const closing = useCloseTile();
  const found = findNote(groups, params);
  if (!found) {
    return null;
  }

  const { machine } = found;
  const close = () => {
    closing.mutate({ machine, tile: params.id });
  };

  const note = found.note.note ?? undefined;
  const title = noteTitle(note);

  return (
    <section
      aria-label={title}
      className="flex h-full min-h-0 flex-col"
      onPointerDown={() => {
        activate(api);
      }}
    >
      <TileBar
        item={{ kind: "note", ...params }}
        active={active}
        icon={<NotebookIcon className="size-4 shrink-0" />}
        title={title}
        onClose={close}
      />
      <div className="min-h-0 flex-1">
        {render({
          machine,
          note,
          onOpen: (next) => {
            show.mutate({ machine, tile: params.id, note: next });
          },
        })}
      </div>
    </section>
  );
}
