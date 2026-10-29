import { GitPullRequestIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import {
  ChecksIcon,
  ReviewIcon,
  StateIcon,
} from "@/features/github/components/rows";
import { checksLabel, reviewLabel } from "@/features/github/lib/labels";
import type { Entry } from "@/features/github/lib/scope";

interface Props {
  branch: string;
  pull: Entry | undefined;
  canOpen: boolean;
  primary: boolean;
  onShow: (entry: Entry) => void;
  onCreate: () => void;
}

export function PullCard({
  branch,
  pull,
  canOpen,
  primary,
  onShow,
  onCreate,
}: Props) {
  if (!pull) {
    if (!canOpen) {
      return null;
    }

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

  const { item } = pull;
  const summary = item.pull;
  const review = summary ? reviewLabel(summary.review) : null;
  const checks = summary?.checks ? checksLabel(summary.checks) : null;

  return (
    <button
      type="button"
      className="flex w-full min-w-0 items-start gap-2 rounded-lg bg-surface-4 p-2 text-left shadow-surface-1 outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring"
      onClick={() => {
        onShow(pull);
      }}
    >
      <span className="flex h-lh shrink-0 items-center type-body">
        <StateIcon
          kind="pull"
          state={item.state}
          draft={summary?.draft ?? false}
        />
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate type-body">
          {item.title}{" "}
          <span className="text-muted-foreground">#{item.number}</span>
        </span>
        <span className="flex flex-wrap items-center gap-x-4 type-caption text-muted-foreground">
          {summary?.draft && <span>Draft</span>}
          {review && (
            <span className="flex items-center gap-2">
              <span aria-hidden className="flex">
                <ReviewIcon review={summary?.review ?? null} />
              </span>
              {review}
            </span>
          )}
          {checks && (
            <span className="flex items-center gap-2">
              <span aria-hidden className="flex">
                <ChecksIcon checks={summary?.checks ?? null} />
              </span>
              {checks}
            </span>
          )}
          {summary && <span>Into {summary.base}</span>}
        </span>
      </span>
    </button>
  );
}
