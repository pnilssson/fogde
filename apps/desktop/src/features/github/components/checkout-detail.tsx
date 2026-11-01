import { useState } from "react";

import { CreatePull } from "@/features/github/components/create-pull";
import { DiffView } from "@/features/github/components/diff-view";
import { LandedCard } from "@/features/github/components/landed-card";
import { NewPull } from "@/features/github/components/new-pull";
import { PullRow } from "@/features/github/components/pull-row";
import { usePlan } from "@/features/github/hooks/use-plan";
import type { Entry } from "@/features/github/lib/scope";
import { select } from "@/features/github/lib/state";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  checkout: CheckoutStatus;
  pull: Entry | undefined;
  merged: Entry | undefined;
  linked: boolean;
  online: boolean;
}

export function CheckoutDetail({
  machine,
  checkout,
  pull,
  merged,
  linked,
  online,
}: Props) {
  const [opening, setOpening] = useState(false);
  const { path, repo, branch } = checkout;
  const { canOpen, next } = usePlan(
    machine,
    checkout,
    pull !== undefined,
    online,
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      {online && !pull && merged && (
        <LandedCard
          machine={machine}
          checkout={checkout}
          entry={merged}
          linked={linked}
        />
      )}
      {online && pull && (
        <PullRow
          pull={pull}
          onShow={(entry) => {
            select(machine.id, {
              kind: "pull",
              repo: entry.repo,
              number: entry.item.number,
            });
          }}
        />
      )}
      {online && branch && !pull && !merged && canOpen && (
        <CreatePull
          branch={branch}
          primary={next === "pull-request"}
          onCreate={() => {
            setOpening(true);
          }}
        />
      )}
      <DiffView
        machine={machine}
        path={path}
        uncommitted={checkout.files.length}
      />
      {repo && branch && (
        <NewPull
          machine={machine}
          repo={repo}
          path={path}
          branch={branch}
          unpushed={checkout.upstream === null || checkout.ahead > 0}
          open={opening}
          onOpenChange={setOpening}
          onCreated={(number) => {
            select(machine.id, { kind: "pull", repo, number });
          }}
        />
      )}
    </div>
  );
}
