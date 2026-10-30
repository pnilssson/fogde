import {
  CheckCircleIcon,
  CircleDashedIcon,
  GitMergeIcon,
  GitPullRequestIcon,
  ProhibitIcon,
  RecordIcon,
  SealCheckIcon,
  XCircleIcon,
} from "@phosphor-icons/react";
import { cn } from "cn";
import type { ReactNode } from "react";

import type { Kind } from "@/features/github/api/github";
import { checksLabel } from "@/features/github/lib/labels";
import { ago } from "@/features/github/lib/scope";
import type {
  ChecksSummary,
  ItemState,
  Listed,
  ReviewState,
} from "@/generated/protocol";

interface RowProps {
  active: boolean;
  icon: ReactNode;
  title: string;
  badges?: ReactNode;
  detail: ReactNode;
  onSelect: () => void;
}

function Row({ active, icon, title, badges, detail, onSelect }: RowProps) {
  return (
    <button
      type="button"
      data-row
      aria-current={active || undefined}
      className={cn(
        "flex w-full min-w-0 items-start gap-2 rounded-md p-2 text-left outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring",
        active && "bg-selected",
      )}
      onClick={onSelect}
    >
      <span className="flex h-lh shrink-0 items-center type-body">{icon}</span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="flex min-w-0 items-center gap-2">
          <span className="min-w-0 flex-1 truncate type-body">{title}</span>
          {badges}
        </span>
        <span className="flex min-w-0 items-center gap-2 type-caption text-muted-foreground">
          {detail}
        </span>
      </span>
    </button>
  );
}

interface ItemRowProps {
  kind: Kind;
  item: Listed;
  active: boolean;
  onSelect: () => void;
}

export function ItemRow({ kind, item, active, onSelect }: ItemRowProps) {
  const { pull } = item;

  return (
    <Row
      active={active}
      icon={
        <StateIcon
          kind={kind}
          state={item.state}
          draft={pull?.draft ?? false}
        />
      }
      title={item.title}
      badges={
        pull && (
          <span className="flex shrink-0 items-center gap-2">
            <ReviewIcon review={pull.review} />
            <ChecksIcon checks={pull.checks} />
          </span>
        )
      }
      detail={
        <>
          <span className="min-w-0 truncate">
            #{item.number}
            {item.author ? ` by ${item.author}` : ""}, {ago(item.updated)}
          </span>
          {pull && (
            <span className="flex shrink-0 gap-2">
              <span className="text-success">+{pull.added}</span>
              <span className="text-destructive">−{pull.removed}</span>
            </span>
          )}
        </>
      }
      onSelect={onSelect}
    />
  );
}

interface StateIconProps {
  kind: Kind;
  state: ItemState;
  draft: boolean;
}

export function StateIcon({ kind, state, draft }: StateIconProps) {
  const tone = cn(
    "size-4",
    state === "open" && !draft ? "text-success" : "text-muted-foreground",
  );
  if (kind === "issue") {
    return state === "open" ? (
      <RecordIcon className={tone} aria-label="Open" />
    ) : (
      <CheckCircleIcon className={tone} aria-label="Closed" />
    );
  }

  return state === "merged" ? (
    <GitMergeIcon className={tone} aria-label="Merged" />
  ) : (
    <GitPullRequestIcon
      className={tone}
      aria-label={state === "closed" ? "Closed" : draft ? "Draft" : "Open"}
    />
  );
}

export function ReviewIcon({ review }: { review: ReviewState | null }) {
  switch (review) {
    case "approved":
      return (
        <SealCheckIcon
          className="size-4 shrink-0 text-success"
          aria-label="Approved"
        />
      );
    case "changes":
      return (
        <ProhibitIcon
          className="size-4 shrink-0 text-destructive"
          aria-label="Changes requested"
        />
      );
    case "required":
    case null:
      return null;
  }
}

export function ChecksIcon({ checks }: { checks: ChecksSummary | null }) {
  if (!checks) {
    return null;
  }

  const label = checksLabel(checks);
  switch (checks.state) {
    case "success":
      return (
        <CheckCircleIcon
          className="size-4 shrink-0 text-success"
          aria-label={label}
        />
      );
    case "failure":
      return (
        <XCircleIcon
          className="size-4 shrink-0 text-destructive"
          aria-label={label}
        />
      );
    case "pending":
      return (
        <CircleDashedIcon
          className="size-4 shrink-0 text-warning"
          aria-label={label}
        />
      );
  }
}
