import { FilePlusIcon, FolderPlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { type ReactNode, useEffect, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { TileStatus } from "@/components/tile-status";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { noteQuery, notesQuery } from "@/features/notes/api/notes";
import { NameDialog } from "@/features/notes/components/name-dialog";
import { NoteEditor } from "@/features/notes/components/note-editor";
import { NotesTree } from "@/features/notes/components/notes-tree";
import { SortMenu } from "@/features/notes/components/sort-menu";
import {
  type Entry,
  useNoteFiles,
} from "@/features/notes/hooks/use-note-files";
import { useListShown } from "@/features/notes/lib/list";
import { view } from "@/features/notes/lib/recent";
import { useOrder } from "@/features/notes/lib/sort";
import { buildTree } from "@/features/notes/lib/tree";
import type { Note } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { report } from "@/lib/report";

interface Props {
  machine: Machine;
  note: string | undefined;
  onOpen: (note: string) => void;
}

export function NotesView({ machine, note, onOpen }: Props) {
  const list = useQuery(notesQuery(machine));
  const order = useOrder();
  const browsing = useListShown();
  const [creating, setCreating] = useState(false);
  const [kind, setKind] = useState<Entry>("note");
  const [parent, setParent] = useState<string>();
  const files = useNoteFiles(machine);
  const tree = useMemo(
    () => list.data && buildTree(list.data, order),
    [list.data, order],
  );

  useReported("Could not open the notes folder", list.error);

  if (list.isError) {
    return (
      <TileStatus
        notice="Could not open the notes folder"
        retry="Try again"
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  }
  if (!list.data || !tree) {
    return <TileStatus notice="Opening notes…" busy />;
  }

  const open = list.data.notes.find(({ path }) => path === note);
  const root = list.data.root;
  const create = (entry: Entry, folder: string) => {
    setKind(entry);
    setParent(folder);
    setCreating(true);
  };

  return (
    <div className="flex h-full min-h-0">
      {browsing && (
        <aside className="flex w-64 min-w-0 shrink-0 flex-col border-r">
          <div className="flex h-8 shrink-0 items-center gap-1 px-2">
            <Action
              label="New note"
              icon={<FilePlusIcon />}
              onClick={() => {
                create("note", open?.folder ?? root);
              }}
            />
            <Action
              label="New folder"
              icon={<FolderPlusIcon />}
              onClick={() => {
                create("folder", open?.folder ?? root);
              }}
            />
            <SortMenu />
          </div>
          <ScrollArea className="min-h-0 flex-1">
            <NotesTree
              machine={machine.id}
              root={tree}
              note={note}
              actions={{
                onOpen,
                onCreate: create,
                onRename: (path, name) =>
                  files.rename.mutateAsync({ path, name }),
                onDelete: (path) => {
                  files.remove.mutate(path);
                },
              }}
            />
          </ScrollArea>
          <NameDialog
            open={creating}
            title={kind === "folder" ? "New folder" : "New note"}
            action="Create"
            busy="Creating…"
            placeholder={kind === "folder" ? "specs" : "Login flow"}
            onOpenChange={setCreating}
            onSubmit={async (name) => {
              const path = await files.create.mutateAsync({
                kind,
                parent: parent ?? root,
                name,
              });
              setCreating(false);
              if (kind === "note") {
                onOpen(path);
              }
            }}
          />
        </aside>
      )}
      <div className="flex min-w-0 flex-1 flex-col">
        {open ? (
          <Shown
            machine={machine}
            note={open}
            notes={list.data.notes}
            onOpen={onOpen}
          />
        ) : files.rename.isPending ? null : note ? (
          <Message
            title="This note was moved or deleted"
            description="Pick another note from the list."
          />
        ) : (
          <Message
            title="No note open"
            description={
              list.data.notes.length > 0
                ? "Pick a note from the list."
                : `Notes you add to ${list.data.root} show here.`
            }
          />
        )}
      </div>
    </div>
  );
}

interface ShownProps {
  machine: Machine;
  note: Note;
  notes: Note[];
  onOpen: (note: string) => void;
}

function Shown({ machine, note, notes, onOpen }: ShownProps) {
  const text = useQuery(noteQuery(machine, note.path));
  useEffect(() => {
    view(machine.id, note.path);
  }, [machine.id, note.path]);

  useReported("Could not read the note", text.error);

  if (text.isError) {
    return (
      <TileStatus
        notice="Could not read the note"
        retry="Try again"
        onRetry={() => {
          void text.refetch();
        }}
      />
    );
  }
  if (text.data === undefined) {
    return <TileStatus notice="Opening note…" busy />;
  }

  return (
    <div className="min-h-0 flex-1">
      <NoteEditor
        key={note.path}
        machine={machine}
        note={note}
        notes={notes}
        loaded={text.data}
        onOpen={onOpen}
      />
    </div>
  );
}

interface ActionProps {
  label: string;
  icon: ReactNode;
  onClick: () => void;
}

function Action({ label, icon, onClick }: ActionProps) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={label}
            onClick={onClick}
          />
        }
      >
        {icon}
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

function useReported(title: string, error: Error | null) {
  useEffect(() => {
    if (error) {
      report(title, error);
    }
  }, [title, error]);
}

function Message({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>{title}</EmptyTitle>
        <EmptyDescription>{description}</EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
