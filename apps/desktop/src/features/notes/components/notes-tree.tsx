import { CaretRightIcon, FolderSimpleIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import { useEffect, useState } from "react";

import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { EntryMenu } from "@/features/notes/components/entry-menu";
import {
  NoteRow,
  type RowActions,
  SubButton,
} from "@/features/notes/components/note-row";
import type { Entry } from "@/features/notes/hooks/use-note-files";
import { type Folder, foldersAbove } from "@/features/notes/lib/tree";
import { moved, onRename } from "@/lib/note-renames";

interface TreeActions extends RowActions {
  onCreate: (kind: Entry, parent: string) => void;
}

interface Props {
  machine: string;
  root: Folder;
  note: string | undefined;
  actions: TreeActions;
}

interface RowsProps {
  folder: Folder;
  nested: boolean;
  note: string | undefined;
  open: Set<string>;
  onToggle: (path: string) => void;
  actions: TreeActions;
}

export function NotesTree({ machine, root, note, actions }: Props) {
  const [open, setOpen] = useState(
    () => new Set(note ? foldersAbove(note, root.path) : []),
  );

  const [shown, setShown] = useState(note);
  if (note !== shown) {
    setShown(note);
    if (note) {
      setOpen(new Set([...open, ...foldersAbove(note, root.path)]));
    }
  }

  useEffect(
    () =>
      onRename((rename) => {
        if (rename.machine !== machine) {
          return;
        }

        setOpen(
          (current) =>
            new Set(
              [...current].map(
                (path) => moved(path, rename.from, rename.to) ?? path,
              ),
            ),
        );
      }),
    [machine],
  );

  const toggle = (path: string) => {
    setOpen((current) => {
      const next = new Set(current);
      if (!next.delete(path)) {
        next.add(path);
      }

      return next;
    });
  };

  return (
    <SidebarMenu>
      <Rows
        folder={root}
        nested={false}
        note={note}
        open={open}
        onToggle={toggle}
        actions={actions}
      />
    </SidebarMenu>
  );
}

function Rows({ folder, nested, note, open, onToggle, actions }: RowsProps) {
  const Item = nested ? SidebarMenuSubItem : SidebarMenuItem;
  const Button = nested ? SubButton : SidebarMenuButton;

  return (
    <>
      {folder.folders.map((inner) => {
        const expanded = open.has(inner.path);

        return (
          <Item key={inner.path}>
            <Collapsible open={expanded}>
              <EntryMenu
                kind="folder"
                name={inner.name}
                path={inner.path}
                trigger={
                  <Button
                    aria-expanded={expanded}
                    onClick={() => {
                      onToggle(inner.path);
                    }}
                  />
                }
                onCreate={(kind) => {
                  if (!expanded) {
                    onToggle(inner.path);
                  }
                  actions.onCreate(kind, inner.path);
                }}
                onRename={actions.onRename}
                onDelete={actions.onDelete}
              >
                <CaretRightIcon
                  className={cn(
                    "transition-transform ease-out motion-reduce:transition-none",
                    expanded && "rotate-90",
                  )}
                />
                <FolderSimpleIcon />
                <span className="min-w-0 flex-1 truncate">{inner.name}</span>
              </EntryMenu>
              <CollapsibleContent>
                {(inner.folders.length > 0 || inner.notes.length > 0) && (
                  <SidebarMenuSub>
                    <Rows
                      folder={inner}
                      nested
                      note={note}
                      open={open}
                      onToggle={onToggle}
                      actions={actions}
                    />
                  </SidebarMenuSub>
                )}
              </CollapsibleContent>
            </Collapsible>
          </Item>
        );
      })}
      {folder.notes.map((current) => (
        <NoteRow
          key={current.path}
          note={current}
          active={current.path === note}
          nested={nested}
          onOpen={actions.onOpen}
          onRename={actions.onRename}
          onDelete={actions.onDelete}
        />
      ))}
    </>
  );
}
