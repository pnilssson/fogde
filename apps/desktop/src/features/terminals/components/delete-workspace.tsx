import { useState } from "react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useDeleteWorkspace } from "@/features/terminals/hooks/use-workspaces";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
}

function outcome(count: number) {
  if (count === 0) {
    return "This removes it from the sidebar.";
  }

  const terminals =
    count === 1 ? "its terminal" : `its ${String(count)} terminals`;

  return `This closes ${terminals} and removes it from the sidebar.`;
}

export function DeleteWorkspace({ machine, workspace }: Props) {
  const [open, setOpen] = useState(false);
  const remove = useDeleteWorkspace(machine, () => {
    setOpen(false);
  });

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogTrigger render={<Button variant="ghost" size="sm" />}>
        Delete workspace
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{`Delete ${workspace.name}?`}</AlertDialogTitle>
          <AlertDialogDescription>
            {`${outcome(workspace.terminals.length)} Your files in `}
            <span className="wrap-anywhere">{workspace.folder}</span>
            {" aren't touched."}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            size="sm"
            disabled={remove.isPending}
            aria-busy={remove.isPending}
            onClick={() => {
              remove.mutate(workspace.id);
            }}
          >
            {remove.isPending && (
              <Spinner aria-hidden data-icon="inline-start" />
            )}
            {remove.isPending ? "Deleting…" : "Delete"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
