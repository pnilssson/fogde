import { GitPullRequestIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";

interface Props {
  branch: string;
  primary: boolean;
  onCreate: () => void;
}

export function CreatePull({ branch, primary, onCreate }: Props) {
  return (
    <div className="flex items-center gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
      <GitPullRequestIcon className="size-4 shrink-0 text-muted-foreground" />
      <p className="min-w-0 flex-1 type-body text-muted-foreground">
        No pull request for {branch} yet.
      </p>
      <Button
        size="sm"
        variant={primary ? "default" : "secondary"}
        onClick={onCreate}
      >
        Create pull request
      </Button>
    </div>
  );
}
