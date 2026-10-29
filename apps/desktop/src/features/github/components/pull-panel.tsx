import {
  ArrowsClockwiseIcon,
  CaretRightIcon,
  CheckCircleIcon,
  CircleDashedIcon,
  MinusCircleIcon,
  XCircleIcon,
} from "@phosphor-icons/react";
import { openUrl } from "@tauri-apps/plugin-opener";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ActionButton } from "@/features/github/components/action-button";
import { CheckoutMenu } from "@/features/github/components/checkout-menu";
import { MergeButton } from "@/features/github/components/merge-button";
import { useWrite } from "@/features/github/hooks/use-write";
import type {
  Check,
  CheckoutStatus,
  Detail,
  MergeState,
  PullDetail,
  Repo,
} from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const states: Record<MergeState, string> = {
  clean: "Ready to merge.",
  behind: "The branch is behind its base.",
  blocked: "Merging is blocked by required reviews or checks.",
  conflicts: "The branch has conflicts to resolve.",
  draft: "This is a draft.",
  unstable: "Some checks did not pass.",
  unknown: "Checking whether it can merge…",
};

interface Props {
  machine: Machine;
  repo: Repo;
  detail: Detail;
  pull: PullDetail;
  checkouts: CheckoutStatus[];
}

export function PullPanel({ machine, repo, detail, pull, checkouts }: Props) {
  const { number } = detail;
  const rerun = useWrite(machine, "Could not re-run the checks");
  const update = useWrite(machine, "Could not update the branch");
  const draft = useWrite(machine, "Could not change the draft state");
  const failed = [
    ...new Set(
      pull.checks
        .filter((check) => check.state === "failure" && check.run !== null)
        .map((check) => check.run),
    ),
  ].filter((run) => run !== null);
  const open = detail.state === "open";

  return (
    <section
      aria-label="Pull request"
      className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1"
    >
      {pull.checks.length > 0 && (
        <div className="flex flex-col">
          <div className="flex items-center gap-2">
            <h3 className="flex-1 type-title">Checks</h3>
            {failed.length > 0 && (
              <ActionButton
                label="Re-run failed checks"
                busy="Re-running…"
                icon={<ArrowsClockwiseIcon data-icon="inline-start" />}
                pending={rerun.isPending}
                onClick={() => {
                  rerun.mutate(async (client) => {
                    for (const run of failed) {
                      await client.call("github.rerun", { repo, run });
                    }
                  });
                }}
              />
            )}
          </div>
          <ul className="flex flex-col">
            {pull.checks.map((check, index) => (
              <CheckRow key={`${check.name}-${String(index)}`} check={check} />
            ))}
          </ul>
        </div>
      )}
      {open && (
        <div className="flex flex-col gap-2">
          <p className="type-body">{states[pull.merge]}</p>
          <div className="flex flex-wrap gap-2">
            {!pull.draft && (
              <MergeButton
                machine={machine}
                repo={repo}
                number={number}
                pull={pull}
              />
            )}
            {pull.draft ? (
              <ActionButton
                variant="default"
                label="Ready for review"
                busy="Marking ready…"
                pending={draft.isPending}
                onClick={() => {
                  draft.mutate((client) =>
                    client.call("github.setDraft", {
                      repo,
                      id: detail.id,
                      draft: false,
                    }),
                  );
                }}
              />
            ) : (
              <ActionButton
                variant="ghost"
                label="Convert to draft"
                busy="Converting…"
                pending={draft.isPending}
                onClick={() => {
                  draft.mutate((client) =>
                    client.call("github.setDraft", {
                      repo,
                      id: detail.id,
                      draft: true,
                    }),
                  );
                }}
              />
            )}
            {pull.merge === "behind" && (
              <ActionButton
                label="Update branch"
                busy="Updating…"
                pending={update.isPending}
                onClick={() => {
                  update.mutate((client) =>
                    client.call("github.updateBranch", { repo, number }),
                  );
                }}
              />
            )}
            <CheckoutMenu
              machine={machine}
              repo={repo}
              number={number}
              pull={pull}
              checkouts={checkouts}
            />
          </div>
        </div>
      )}
      <Collapsible>
        <CollapsibleTrigger className="group flex items-center gap-2 rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring">
          <CaretRightIcon className="size-4 transition-transform ease-out group-data-[panel-open]:rotate-90 motion-reduce:transition-none" />
          <span className="type-body">
            {pull.changed === 1
              ? "1 file changed"
              : `${String(pull.changed)} files changed`}
          </span>
          <span className="type-body text-success">+{pull.added}</span>
          <span className="type-body text-destructive">−{pull.removed}</span>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <ul className="flex flex-col pl-4">
            {pull.files.map((file) => (
              <li key={file.path} className="flex min-w-0 items-center gap-2">
                <span className="min-w-0 flex-1 truncate type-code">
                  {file.path}
                </span>
                <span className="type-caption text-success">+{file.added}</span>
                <span className="type-caption text-destructive">
                  −{file.removed}
                </span>
              </li>
            ))}
          </ul>
        </CollapsibleContent>
      </Collapsible>
    </section>
  );
}

function CheckRow({ check }: { check: Check }) {
  const content = (
    <>
      <CheckIcon state={check.state} />
      <span className="min-w-0 truncate type-body">{check.name}</span>
    </>
  );

  return (
    <li>
      {check.url ? (
        <button
          type="button"
          className="flex h-8 w-full min-w-0 items-center gap-2 rounded-md px-2 text-left outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring"
          onClick={() => {
            if (check.url) {
              void openUrl(check.url);
            }
          }}
        >
          {content}
        </button>
      ) : (
        <div className="flex h-8 min-w-0 items-center gap-2 px-2">
          {content}
        </div>
      )}
    </li>
  );
}

function CheckIcon({ state }: { state: Check["state"] }) {
  switch (state) {
    case "success":
      return (
        <CheckCircleIcon
          className="size-4 shrink-0 text-success"
          aria-label="Passed"
        />
      );
    case "failure":
      return (
        <XCircleIcon
          className="size-4 shrink-0 text-destructive"
          aria-label="Failed"
        />
      );
    case "pending":
      return (
        <CircleDashedIcon
          className="size-4 shrink-0 text-warning"
          aria-label="Running"
        />
      );
    case "skipped":
    case "neutral":
      return (
        <MinusCircleIcon
          className="size-4 shrink-0 text-muted-foreground"
          aria-label="Skipped"
        />
      );
  }
}
