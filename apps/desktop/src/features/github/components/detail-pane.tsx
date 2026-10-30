import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { CheckoutDetail } from "@/features/github/components/checkout-detail";
import { ItemDetail } from "@/features/github/components/item-detail";
import { type Entry, landed, pullFor } from "@/features/github/lib/scope";
import { type Item, sameRepo, type Tab } from "@/features/github/lib/state";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  tab: Tab;
  checkout: CheckoutStatus | undefined;
  item: Item | undefined;
  checkouts: CheckoutStatus[];
  openPulls: Entry[];
  closedPulls: Entry[];
  linked: boolean;
  online: boolean;
}

export function DetailPane({
  machine,
  tab,
  checkout,
  item,
  checkouts,
  openPulls,
  closedPulls,
  linked,
  online,
}: Props) {
  if (tab === "changes") {
    if (!checkout) {
      return (
        <Nothing
          title="No checkout"
          hint="Open a workspace with a Git repository to see its changes."
        />
      );
    }

    return (
      <CheckoutDetail
        key={checkout.path}
        machine={machine}
        checkout={checkout}
        pull={pullFor(openPulls, checkout)}
        merged={landed(closedPulls, checkout)}
        linked={linked}
        online={online}
      />
    );
  }

  if (!item) {
    return (
      <Nothing
        title="Nothing selected"
        hint={
          tab === "pull"
            ? "Pick a pull request from the list."
            : "Pick an issue from the list."
        }
      />
    );
  }

  return (
    <ItemDetail
      key={`${item.kind}/${item.repo.owner}/${item.repo.name}#${String(item.number)}`}
      machine={machine}
      kind={item.kind}
      repo={item.repo}
      number={item.number}
      checkouts={checkouts.filter(
        (candidate) => candidate.repo && sameRepo(candidate.repo, item.repo),
      )}
    />
  );
}

function Nothing({ title, hint }: { title: string; hint: string }) {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>{title}</EmptyTitle>
        <EmptyDescription>{hint}</EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
