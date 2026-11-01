import { EditorView } from "@codemirror/view";
import { useQueryClient } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useRef } from "react";

import { ScrollArea } from "@/components/ui/scroll-area";
import { toast } from "@/components/ui/toast";
import { noteQuery } from "@/features/notes/api/notes";
import { NoteTitle } from "@/features/notes/components/note-title";
import { noteState, replaceText } from "@/features/notes/lib/editor";
import { resolveHref, resolveWikilink } from "@/features/notes/lib/links";
import { NoteSession, resume } from "@/features/notes/lib/session";
import type { Note, NoteText } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { report } from "@/lib/report";

interface Props {
  machine: Machine;
  note: Note;
  notes: Note[];
  loaded: NoteText;
  onOpen: (path: string) => void;
  onRename: (name: string) => Promise<unknown>;
}

export function NoteEditor({
  machine,
  note,
  notes,
  loaded,
  onOpen,
  onRename,
}: Props) {
  const queries = useQueryClient();
  const container = useRef<HTMLDivElement>(null);
  const session = useRef<NoteSession>(undefined);
  const editor = useRef<EditorView>(undefined);
  const links = useRef({ note, notes, onOpen });
  const latest = useRef(loaded);
  const { id, client } = machine;

  useEffect(() => {
    links.current = { note, notes, onOpen };
  }, [note, notes, onOpen]);

  useEffect(() => {
    const element = container.current;
    if (!element) {
      return;
    }

    const query = noteQuery({ id, client }, note.path);
    const view = new EditorView({ parent: element });
    const key = `${id}:${note.path}`;
    const start = resume(key) ?? {
      base: latest.current.text,
      version: latest.current.version,
      text: latest.current.text,
    };
    const current = new NoteSession(
      start.base,
      start.version,
      {
        text: () => view.state.sliceDoc(),
        replace: (text) => {
          replaceText(view, text);
        },
      },
      {
        write: (text, version) =>
          client.call("notes.write", {
            path: note.path,
            text,
            version,
          }),
        read: () => queries.query({ ...query, staleTime: 0 }),
        saved: (text, version) => {
          queries.setQueryData(query.queryKey, { text, version });
        },
        kept: () => {
          toast.add({
            type: "warning",
            title: "Kept your version of the note",
            description:
              "The note changed on disk where you were editing, and your edit replaced that change.",
          });
        },
        failed: (error) => {
          report("Could not save the note", error);
        },
      },
      key,
    );
    view.setState(
      noteState(
        start.text,
        {
          wikilink: (target) => {
            const { note: from, notes: all, onOpen: open } = links.current;
            const found = resolveWikilink(target, all, from);
            if (found) {
              open(found.path);
            }
          },
          link: (href) => {
            const { note: from, notes: all, onOpen: open } = links.current;
            const found = resolveHref(href, all, from);
            if (found) {
              open(found.path);
            } else if (/^(https?|mailto):/i.test(href)) {
              openUrl(href).catch((error: unknown) => {
                report("Could not open the link", error);
              });
            }
          },
        },
        () => {
          current.edited();
        },
      ),
    );
    session.current = current;
    editor.current = view;
    if (start.text !== start.base) {
      current.edited();
    }
    const left = () => {
      current.flush();
    };
    view.contentDOM.addEventListener("blur", left);
    if (start.text === "") {
      view.focus();
    }

    return () => {
      view.contentDOM.removeEventListener("blur", left);
      current.close();
      view.destroy();
      session.current = undefined;
      editor.current = undefined;
    };
  }, [id, client, note.path, queries]);

  useEffect(() => {
    latest.current = loaded;
    session.current?.loaded(loaded.text, loaded.version);
  }, [loaded]);

  return (
    <ScrollArea className="h-full">
      <div className="flex min-h-full flex-col">
        <NoteTitle
          name={note.name}
          onRename={onRename}
          onDone={() => {
            editor.current?.focus();
          }}
        />
        <div
          ref={container}
          className="flex flex-1 flex-col select-text"
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
        />
      </div>
    </ScrollArea>
  );
}
