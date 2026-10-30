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
import { NoteSession } from "@/features/notes/lib/session";
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

  useEffect(() => {
    links.current = { note, notes, onOpen };
  }, [note, notes, onOpen]);

  useEffect(() => {
    const element = container.current;
    if (!element) {
      return;
    }

    const query = noteQuery(machine, note.path);
    const view = new EditorView({ parent: element });
    const current = new NoteSession(
      latest.current.text,
      latest.current.version,
      {
        text: () => view.state.sliceDoc(),
        replace: (text) => {
          replaceText(view, text);
        },
      },
      {
        write: (text, version) =>
          machine.client.call("notes.write", {
            path: note.path,
            text,
            version,
          }),
        saved: (text, version) => {
          queries.setQueryData(query.queryKey, { text, version });
        },
        reload: () => {
          void queries.invalidateQueries(query);
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
    );
    view.setState(
      noteState(
        latest.current.text,
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
    const left = () => {
      current.flush();
    };
    view.contentDOM.addEventListener("blur", left);
    if (latest.current.text === "") {
      view.focus();
    }

    return () => {
      view.contentDOM.removeEventListener("blur", left);
      current.flush();
      view.destroy();
      session.current = undefined;
      editor.current = undefined;
    };
  }, [machine, note.path, queries]);

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
