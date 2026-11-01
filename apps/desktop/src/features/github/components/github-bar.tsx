import { ColumnsIcon, RowsIcon } from "@phosphor-icons/react";
import type { ReactNode } from "react";

import { Choice } from "@/components/choice";
import { PageBar } from "@/components/page-bar";
import { BranchMenu } from "@/features/github/components/branch-menu";
import {
  CheckoutPicker,
  type Group,
} from "@/features/github/components/checkout-picker";
import {
  FetchButton,
  SyncButton,
} from "@/features/github/components/sync-button";
import { usePlan } from "@/features/github/hooks/use-plan";
import { useOpenCheckout } from "@/features/github/lib/open";
import { type Entry, pullFor } from "@/features/github/lib/scope";
import {
  setDiffStyle,
  useDiffStyle,
  useGitHubState,
} from "@/features/github/lib/state";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const styles = [
  { value: "unified", label: "Unified", icon: RowsIcon },
  { value: "split", label: "Split", icon: ColumnsIcon },
] as const;

interface Props {
  machine: Machine;
  groups: Group[];
  checkout: CheckoutStatus | undefined;
  openPulls: Entry[];
  online: boolean;
  picker: ReactNode;
}

export function GitHubBar({
  machine,
  groups,
  checkout,
  openPulls,
  online,
  picker,
}: Props) {
  const open = useOpenCheckout();
  const { tab } = useGitHubState();
  const style = useDiffStyle();
  const changes = tab === "changes" && checkout !== undefined;

  return (
    <PageBar>
      <CheckoutPicker
        groups={groups}
        chosen={checkout}
        openPulls={openPulls}
        onPick={(path) => {
          open(path);
        }}
      />
      {checkout && (
        <CheckoutControls
          machine={machine}
          checkout={checkout}
          hasPull={pullFor(openPulls, checkout) !== undefined}
          online={online}
          onOpen={open}
        />
      )}
      <span className="flex-1" />
      {changes && (
        <Choice
          label="Layout"
          value={style}
          options={styles}
          onChange={setDiffStyle}
        />
      )}
      {picker}
    </PageBar>
  );
}

interface ControlsProps {
  machine: Machine;
  checkout: CheckoutStatus;
  hasPull: boolean;
  online: boolean;
  onOpen: (path: string) => void;
}

function CheckoutControls({
  machine,
  checkout,
  hasPull,
  online,
  onOpen,
}: ControlsProps) {
  const { sync, next } = usePlan(machine, checkout, hasPull, online);

  return (
    <>
      <BranchMenu
        machine={machine}
        path={checkout.path}
        branch={checkout.branch}
        onOpen={onOpen}
      />
      <FetchButton machine={machine} checkout={checkout} />
      <SyncButton
        machine={machine}
        checkout={checkout}
        sync={sync}
        primary={next === "sync"}
      />
    </>
  );
}
