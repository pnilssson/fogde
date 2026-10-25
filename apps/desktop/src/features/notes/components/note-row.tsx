import { FileTextIcon } from "@phosphor-icons/react";
import { type ComponentProps, useState } from "react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { NameDialog } from "@/features/notes/components/name-dialog";
import type { Note } from "@/generated/protocol";
import { noteDragProps } from "@/lib/note-drag";
import { report } from "@/lib/report";

export interface RowActions {
  onOpen: (path: string) => void;
  onRename: (path: string, name: string) => Promise<unknown>;
  onDelete: (path: string) => void;
}

interface Props extends RowActions {
  machine: string;
  note: Note;
  active: boolean;
  nested: boolean;
}

function copy(path: string) {
  navigator.clipboard.writeText(path).catch((error: unknown) => {
    report("Could not copy the path", error);
  });
}

export function NoteRow({
  machine,
  note,
  active,
  nested,
  onOpen,
  onRename,
  onDelete,
}: Props) {
  const [renaming, setRenaming] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const Item = nested ? SidebarMenuSubItem : SidebarMenuItem;
  const Button = nested ? SubButton : SidebarMenuButton;

  return (
    <Item>
      <ContextMenu>
        <ContextMenuTrigger
          render={
            <Button
              isActive={active}
              {...noteDragProps({ machine, path: note.path })}
              onClick={() => {
                onOpen(note.path);
              }}
            />
          }
        >
          <FileTextIcon />
          <span className="min-w-0 flex-1 truncate">{note.name}</span>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem
            onClick={() => {
              copy(note.path);
            }}
          >
            Copy path
          </ContextMenuItem>
          <ContextMenuItem
            onClick={() => {
              setRenaming(true);
            }}
          >
            Rename
          </ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem
            variant="destructive"
            onClick={() => {
              setDeleting(true);
            }}
          >
            Delete
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      <NameDialog
        open={renaming}
        title={`Rename ${note.name}`}
        action="Rename"
        busy="Renaming…"
        placeholder="Note name"
        initial={note.name}
        onOpenChange={setRenaming}
        onSubmit={async (name) => {
          await onRename(note.path, name);
          setRenaming(false);
        }}
      />
      <AlertDialog open={deleting} onOpenChange={setDeleting}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{`Delete ${note.name}?`}</AlertDialogTitle>
            <AlertDialogDescription>
              It moves to the .trash folder in your notes folder.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              size="sm"
              onClick={() => {
                onDelete(note.path);
                setDeleting(false);
              }}
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Item>
  );
}

function SubButton(props: ComponentProps<typeof SidebarMenuSubButton>) {
  return <SidebarMenuSubButton render={<button type="button" />} {...props} />;
}
