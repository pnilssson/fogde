import { NotebookIcon } from "@phosphor-icons/react";
import type { IDockviewPanelProps } from "dockview-react";
import { useCallback, useContext } from "react";

import { TileBar } from "@/features/workspaces/components/tile-bar";
import {
  NotesParams,
  noteTitle,
  useTiles,
} from "@/features/workspaces/lib/layouts";
import { showNote } from "@/features/workspaces/lib/opening";
import { WorkspaceContext } from "@/features/workspaces/lib/selection";
import { NotesSlot } from "@/features/workspaces/lib/slot";
import { activate } from "@/features/workspaces/lib/tiles";

export function NotesPane({ api, params }: IDockviewPanelProps) {
  const render = useContext(NotesSlot);
  const { groups } = useContext(WorkspaceContext);
  const active = useTiles((tiles) => tiles.focused === api.id);
  const close = useCallback(() => {
    api.close();
  }, [api]);
  const parsed = NotesParams.safeParse(params).data;
  const machine = groups.find(
    (group) => group.machine.id === parsed?.machine,
  )?.machine;
  if (!parsed || !machine) {
    return null;
  }

  const title = noteTitle(parsed.note);

  return (
    <section
      aria-label={title}
      className="flex h-full min-h-0 flex-col"
      onPointerDown={() => {
        activate(api);
      }}
    >
      <TileBar
        item={{ tile: api.id }}
        active={active}
        icon={<NotebookIcon className="size-4 shrink-0" />}
        title={title}
        onClose={close}
      />
      <div className="min-h-0 flex-1">
        {render({
          machine,
          note: parsed.note,
          onOpen: (note) => {
            showNote(api, { ...parsed, note });
          },
        })}
      </div>
    </section>
  );
}
