import { FilePlusIcon, FolderPlusIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { type ReactNode, useEffect, useMemo, useState } from "react";

import { Bar, Body, Layer, Main, SideScroll } from "@/components/frame";
import { PageShell } from "@/components/shell";
import { IconAction } from "@/components/icon-action";
import { PageBar } from "@/components/page-bar";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { TileStatus } from "@/components/tile-status";
import { noteQuery, notesQuery } from "@/features/notes/api/notes";
import { NameDialog } from "@/features/notes/components/name-dialog";
import { NoteEditor } from "@/features/notes/components/note-editor";
import { NotesTree } from "@/features/notes/components/notes-tree";
import { SortMenu } from "@/features/notes/components/sort-menu";
import {
  type Entry,
  useNoteFiles,
} from "@/features/notes/hooks/use-note-files";
import { view } from "@/features/notes/lib/recent";
import { useOrder } from "@/features/notes/lib/sort";
import { buildTree } from "@/features/notes/lib/tree";
import type { Note } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { report } from "@/lib/report";

interface Props {
  machine: Machine;
  note: string | undefined;
  picker: ReactNode;
  onOpen: (note: string) => void;
}

export function NotesView({ machine, note, picker, onOpen }: Props) {
  const list = useQuery(notesQuery(machine));
  const order = useOrder();
  const [creating, setCreating] = useState(false);
  const [kind, setKind] = useState<Entry>("note");
  const [parent, setParent] = useState<string>();
  const files = useNoteFiles(machine);
  const tree = useMemo(
    () => list.data && buildTree(list.data, order),
    [list.data, order],
  );

  useReported("Could not open the notes folder", list.error);

  if (list.isError || !list.data || !tree) {
    return (
      <PageShell>
        <PageBar>
          <span className="flex-1" />
          {picker}
        </PageBar>
        {list.isError ? (
          <TileStatus
            notice="Could not open the notes folder"
            retry="Try again"
            fill
            onRetry={() => {
              void list.refetch();
            }}
          />
        ) : (
          <TileStatus notice="Opening notes…" busy fill />
        )}
      </PageShell>
    );
  }

  const open = list.data.notes.find(({ path }) => path === note);
  const root = list.data.root;
  const create = (entry: Entry, folder: string) => {
    setKind(entry);
    setParent(folder);
    setCreating(true);
  };

  return (
    <PageShell
      side={
        <Layer aria-label="Notes list" className="flex-1">
          <Bar>
            <IconAction
              label="New note"
              icon={<FilePlusIcon />}
              onClick={() => {
                create("note", open?.folder ?? root);
              }}
            />
            <IconAction
              label="New folder"
              icon={<FolderPlusIcon />}
              onClick={() => {
                create("folder", open?.folder ?? root);
              }}
            />
            <SortMenu />
          </Bar>
          <SideScroll>
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
          </SideScroll>
        </Layer>
      }
    >
      <PageBar>
        <span className="flex-1" />
        {picker}
      </PageBar>
      <Body>
        <Main>
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
        </Main>
      </Body>
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
    </PageShell>
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
        fill
        onRetry={() => {
          void text.refetch();
        }}
      />
    );
  }
  if (text.data === undefined) {
    return <TileStatus notice="Opening note…" busy fill />;
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
