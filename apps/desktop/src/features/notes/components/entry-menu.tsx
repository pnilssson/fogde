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
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { NameDialog } from "@/features/notes/components/name-dialog";
import type { Entry } from "@/features/notes/hooks/use-note-files";
import { report } from "@/lib/report";

export interface EntryActions {
  onRename: (path: string, name: string) => Promise<unknown>;
  onDelete: (path: string) => void;
  onInsert?: ((path: string) => void) | undefined;
}

interface Props extends EntryActions {
  kind: Entry;
  name: string;
  path: string;
  trigger: ReactElement;
  children: ReactNode;
  onCreate?: (kind: Entry) => void;
}

function copy(path: string) {
  navigator.clipboard.writeText(path).catch((error: unknown) => {
    report("Could not copy the path", error);
  });
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
  onInsert,
}: Props) {
  const [renaming, setRenaming] = useState(false);
  const [deleting, setDeleting] = useState(false);

  return (
    <>
      <ContextMenu>
        <ContextMenuTrigger render={trigger}>{children}</ContextMenuTrigger>
        <ContextMenuContent>
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
          {onInsert && (
            <ContextMenuItem
              onClick={() => {
                onInsert(path);
              }}
            >
              Insert path
            </ContextMenuItem>
          )}
          <ContextMenuItem
            onClick={() => {
              copy(path);
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
