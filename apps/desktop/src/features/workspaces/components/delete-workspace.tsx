import { ConfirmDialog } from "@/components/confirm-dialog";
import { useDeleteWorkspace } from "@/features/workspaces/hooks/use-workspaces";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspace: Workspace;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function outcome(count: number) {
  if (count === 0) {
    return "This removes it from the sidebar.";
  }

  const terminals =
    count === 1 ? "its terminal" : `its ${String(count)} terminals`;

  return `This closes ${terminals} and removes it from the sidebar.`;
}

export function DeleteWorkspace({
  machine,
  workspace,
  open,
  onOpenChange,
}: Props) {
  const remove = useDeleteWorkspace(machine, () => {
    onOpenChange(false);
  });

  return (
    <ConfirmDialog
      open={open}
      onOpenChange={onOpenChange}
      title={`Delete ${workspace.name}?`}
      description={
        <>
          {`${outcome(workspace.terminals.length)} Your files in `}
          <span className="wrap-anywhere">{workspace.folder}</span>
          {" aren't touched."}
        </>
      }
      action="Delete"
      busy="Deleting…"
      pending={remove.isPending}
      onConfirm={() => {
        remove.mutate(workspace.id);
      }}
    />
  );
}
