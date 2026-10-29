import type { ReactNode } from "react";

import { Choice } from "@/components/choice";
import { PageBar } from "@/components/page-bar";
import { BranchMenu } from "@/features/github/components/branch-menu";
import { CheckoutPicker } from "@/features/github/components/checkout-picker";
import { SyncButton } from "@/features/github/components/sync-button";
import { usePlan } from "@/features/github/hooks/use-plan";
import { type Entry, pullFor } from "@/features/github/lib/scope";
import { type Scope, select, setScope } from "@/features/github/lib/state";
import type { CheckoutStatus, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  current: Workspace | undefined;
  scope: Scope;
  checkouts: CheckoutStatus[];
  checkout: CheckoutStatus | undefined;
  openPulls: Entry[];
  online: boolean;
  picker: ReactNode;
}

export function GitHubBar({
  machine,
  current,
  scope,
  checkouts,
  checkout,
  openPulls,
  online,
  picker,
}: Props) {
  return (
    <PageBar>
      <CheckoutPicker
        checkouts={checkouts}
        chosen={checkout}
        openPulls={openPulls}
        onPick={(path) => {
          select(machine.id, { kind: "checkout", path });
        }}
      />
      {checkout && (
        <CheckoutControls
          machine={machine}
          checkout={checkout}
          checkouts={checkouts}
          hasPull={pullFor(openPulls, checkout) !== undefined}
          online={online}
        />
      )}
      <span className="flex-1" />
      {current && (
        <Choice
          label="Scope"
          value={scope}
          options={[
            { value: "workspace", label: current.name },
            { value: "all", label: "All workspaces" },
          ]}
          onChange={setScope}
        />
      )}
      {picker}
    </PageBar>
  );
}

interface ControlsProps {
  machine: Machine;
  checkout: CheckoutStatus;
  checkouts: CheckoutStatus[];
  hasPull: boolean;
  online: boolean;
}

function CheckoutControls({
  machine,
  checkout,
  checkouts,
  hasPull,
  online,
}: ControlsProps) {
  const { sync, next } = usePlan(machine, checkout, hasPull, online);

  return (
    <>
      <BranchMenu
        machine={machine}
        path={checkout.path}
        branch={checkout.branch}
        checkouts={checkouts.map(({ path }) => path)}
        onOpen={(other) => {
          select(machine.id, { kind: "checkout", path: other });
        }}
      />
      <SyncButton
        machine={machine}
        checkout={checkout}
        sync={sync}
        primary={next === "sync"}
      />
    </>
  );
}
