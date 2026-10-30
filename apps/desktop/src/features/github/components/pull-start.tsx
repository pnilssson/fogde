import { PlusIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { NewPull } from "@/features/github/components/new-pull";
import { usePlan } from "@/features/github/hooks/use-plan";
import { select } from "@/features/github/lib/state";
import type { CheckoutStatus, Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  checkout: CheckoutStatus;
  repo: Repo;
}

export function PullStart({ machine, checkout, repo }: Props) {
  const [open, setOpen] = useState(false);
  const { canOpen } = usePlan(machine, checkout, false, true);
  const { branch } = checkout;
  if (!canOpen || !branch) {
    return null;
  }

  return (
    <>
      <Button
        variant="ghost"
        size="sm"
        className="justify-start"
        onClick={() => {
          setOpen(true);
        }}
      >
        <PlusIcon data-icon="inline-start" />
        New pull request
      </Button>
      <NewPull
        machine={machine}
        repo={repo}
        path={checkout.path}
        branch={branch}
        unpushed={checkout.upstream === null || checkout.ahead > 0}
        open={open}
        onOpenChange={setOpen}
        onCreated={(number) => {
          select(machine.id, { kind: "pull", repo, number });
        }}
      />
    </>
  );
}
