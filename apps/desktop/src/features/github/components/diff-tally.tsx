import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { Diff, DiffScope } from "@/generated/protocol";

interface Props {
  diff: Diff;
  scope: DiffScope;
  branch: string | null;
}

export function DiffTally({ diff, scope, branch }: Props) {
  const count = diff.files.length;
  if (count === 0) {
    return null;
  }

  const added = diff.files.reduce((sum, file) => sum + file.added, 0);
  const removed = diff.files.reduce((sum, file) => sum + file.removed, 0);
  const files = `${String(count)} ${count === 1 ? "file" : "files"}`;
  const label =
    scope === "uncommitted" ? null : diff.base ? `vs ${diff.base}` : null;

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <span className="flex min-w-0 items-center gap-2 font-mono type-caption tabular-nums" />
        }
      >
        <span className="shrink-0">{files}</span>
        <span className="shrink-0 text-success">+{added}</span>
        <span className="shrink-0 text-destructive">−{removed}</span>
        {label && (
          <span className="truncate text-muted-foreground">{label}</span>
        )}
      </TooltipTrigger>
      <TooltipContent>
        {scope === "uncommitted" || !diff.base
          ? `${files} you haven't committed yet`
          : `${files} that ${branch ?? "this branch"} changes compared with ${diff.base}, committed or not`}
      </TooltipContent>
    </Tooltip>
  );
}
