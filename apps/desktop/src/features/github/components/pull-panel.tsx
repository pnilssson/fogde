import { CaretRightIcon } from "@phosphor-icons/react";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ActionButton } from "@/features/github/components/action-button";
import { CheckList } from "@/features/github/components/check-list";
import { CheckoutMenu } from "@/features/github/components/checkout-menu";
import { MergeButton } from "@/features/github/components/merge-button";
import { useWrite } from "@/features/github/hooks/use-write";
import { blockers } from "@/features/github/lib/blockers";
import type {
  CheckoutStatus,
  Detail,
  MergeState,
  PullDetail,
  Repo,
} from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const states: Partial<Record<MergeState, string>> = {
  clean: "Ready to merge.",
  unstable: "Some checks did not pass, but it can merge.",
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
  const update = useWrite(machine, "Could not update the branch");
  const draft = useWrite(machine, "Could not change the draft state");
  const open = detail.state === "open";

  return (
    <section
      aria-label="Pull request"
      className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1"
    >
      {pull.checks.length > 0 && (
        <CheckList machine={machine} repo={repo} checks={pull.checks} />
      )}
      {open && (
        <div className="flex flex-col gap-2">
          <Status pull={pull} />
          <div className="flex flex-wrap gap-2">
            {!pull.draft && (
              <MergeButton
                machine={machine}
                repo={repo}
                id={detail.id}
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

function Status({ pull }: { pull: PullDetail }) {
  const reasons = blockers(pull);
  if (reasons.length === 0) {
    return (
      <p className="type-body">
        {pull.auto ? "Merging automatically…" : states[pull.merge]}
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-2">
      <p className="type-body">
        {pull.auto
          ? "GitHub merges it automatically once these are done:"
          : "Before it can merge:"}
      </p>
      <ul className="flex list-disc flex-col gap-2 pl-8 type-body">
        {reasons.map((reason) => (
          <li key={reason}>{reason}</li>
        ))}
      </ul>
    </div>
  );
}
