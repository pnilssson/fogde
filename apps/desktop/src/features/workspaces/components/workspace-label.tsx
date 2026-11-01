import { cn } from "cn";

import { BranchStats } from "@/features/workspaces/components/branch-stats";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { changesOf } from "@/features/workspaces/lib/checkouts";
import { attention } from "@/features/workspaces/lib/status";
import type { Checkout, Workspace } from "@/generated/protocol";

interface Props {
  workspace: Workspace;
  main: Checkout | undefined;
  status: boolean;
}

export function WorkspaceLabel({ workspace, main, status }: Props) {
  return (
    <span className="flex min-w-0 flex-1 flex-col">
      <span className="flex min-w-0 items-center gap-2">
        <span
          className={cn(
            "min-w-0 flex-1 truncate",
            workspace.missing && "text-muted-foreground",
          )}
        >
          {workspace.name}
        </span>
        {status && <StateIcon status={attention(workspace.terminals)} />}
      </span>
      {main?.branch && (
        <span className="flex min-w-0 items-center gap-2 font-mono type-caption text-muted-foreground">
          <span className="min-w-0 truncate">{main.branch}</span>
          <BranchStats changes={changesOf(workspace, main)} />
        </span>
      )}
    </span>
  );
}
