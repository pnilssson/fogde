import { NameDialog } from "@/components/name-dialog";
import { useWrite } from "@/features/github/hooks/use-write";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  path: string;
  from: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewBranch({ machine, path, from, open, onOpenChange }: Props) {
  const create = useWrite(machine, "Could not create the branch");

  return (
    <NameDialog
      open={open}
      title="New branch"
      description={
        from
          ? `Starts from ${from} and takes your uncommitted changes along.`
          : "Starts here and takes your uncommitted changes along."
      }
      action="Create"
      busy="Creating…"
      placeholder="feature/login"
      onOpenChange={onOpenChange}
      onSubmit={async (branch) => {
        await create.mutateAsync((client) =>
          client.call("checkouts.createBranch", { path, branch }),
        );
        onOpenChange(false);
      }}
    />
  );
}
