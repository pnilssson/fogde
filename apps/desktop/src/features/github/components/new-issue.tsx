import { TextDialog } from "@/features/github/components/text-dialog";
import { useWrite } from "@/features/github/hooks/use-write";
import type { Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  repo: Repo;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (number: number) => void;
}

export function NewIssue({
  machine,
  repo,
  open,
  onOpenChange,
  onCreated,
}: Props) {
  const create = useWrite<number>(machine, "Could not create the issue");

  return (
    <TextDialog
      open={open}
      heading={`New issue in ${repo.owner}/${repo.name}`}
      action="Create"
      busy="Creating…"
      onOpenChange={onOpenChange}
      onSubmit={async ({ title, body }) => {
        const number = await create.mutateAsync((client) =>
          client.call("github.newIssue", { repo, title, body }),
        );
        onOpenChange(false);
        onCreated(number);
      }}
    />
  );
}
