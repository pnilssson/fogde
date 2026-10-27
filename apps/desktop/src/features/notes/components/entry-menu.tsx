import { type ReactElement, type ReactNode, useState } from "react";

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
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Keys } from "@/components/keys";
import { NameDialog } from "@/features/notes/components/name-dialog";
import type { Entry } from "@/features/notes/hooks/use-note-files";
import { copyPath } from "@/features/notes/lib/copy";
import { pressed, shortcuts } from "@/lib/shortcuts";

export interface EntryActions {
  onRename: (path: string, name: string) => Promise<unknown>;
  onDelete: (path: string) => void;
}

interface Props extends EntryActions {
  kind: Entry;
  name: string;
  path: string;
  trigger: ReactElement;
  children: ReactNode;
  onCreate?: (kind: Entry) => void;
}

export function EntryMenu({
  kind,
  name,
  path,
  trigger,
  children,
  onCreate,
  onRename,
  onDelete,
}: Props) {
  const [open, setOpen] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [deleting, setDeleting] = useState(false);

  return (
    <>
      <ContextMenu open={open} onOpenChange={setOpen}>
        <ContextMenuTrigger render={trigger}>{children}</ContextMenuTrigger>
        <ContextMenuContent
          onKeyDown={(event) => {
            if (!pressed(event.nativeEvent, shortcuts.path)) {
              return;
            }

            event.preventDefault();
            copyPath(path);
            setOpen(false);
          }}
        >
          {onCreate && (
            <>
              <ContextMenuItem
                onClick={() => {
                  onCreate("note");
                }}
              >
                New note
              </ContextMenuItem>
              <ContextMenuItem
                onClick={() => {
                  onCreate("folder");
                }}
              >
                New folder
              </ContextMenuItem>
              <ContextMenuSeparator />
            </>
          )}
          <ContextMenuItem
            onClick={() => {
              copyPath(path);
            }}
          >
            Copy path
            <ContextMenuShortcut>
              <Keys letter={shortcuts.path} />
            </ContextMenuShortcut>
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
        title={`Rename ${name}`}
        action="Rename"
        busy="Renaming…"
        placeholder={kind === "folder" ? "Folder name" : "Note name"}
        initial={name}
        onOpenChange={setRenaming}
        onSubmit={async (next) => {
          await onRename(path, next);
          setRenaming(false);
        }}
      />
      <AlertDialog open={deleting} onOpenChange={setDeleting}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{`Delete ${name}?`}</AlertDialogTitle>
            <AlertDialogDescription>
              {kind === "folder"
                ? "It moves with everything in it to the .trash folder in your notes folder."
                : "It moves to the .trash folder in your notes folder."}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              size="sm"
              onClick={() => {
                onDelete(path);
                setDeleting(false);
              }}
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
