import { CloudArrowDownIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useWrite } from "@/features/github/hooks/use-write";
import { ago } from "@/features/github/lib/scope";
import type { Sync } from "@/features/github/lib/sync";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface FetchProps {
  machine: Machine;
  checkout: CheckoutStatus;
}

export function FetchButton({ machine, checkout }: FetchProps) {
  const write = useWrite(machine, "Could not fetch");
  const fetched =
    checkout.fetched === null
      ? "never fetched"
      : `last fetched ${ago(checkout.fetched)}`;

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label="Fetch"
            disabled={write.isPending}
            aria-busy={write.isPending}
            onClick={() => {
              write.mutate((client) =>
                client.call("checkouts.fetch", checkout.path),
              );
            }}
          />
        }
      >
        {write.isPending ? <Spinner aria-hidden /> : <CloudArrowDownIcon />}
      </TooltipTrigger>
      <TooltipContent>Fetch, {fetched}</TooltipContent>
    </Tooltip>
  );
}

interface SyncProps {
  machine: Machine;
  checkout: CheckoutStatus;
  sync: Sync;
  primary: boolean;
}

type Action = "push" | "pull";

export function SyncButton({ machine, checkout, sync, primary }: SyncProps) {
  const write = useWrite(machine, failures[sync.kind]);
  if (sync.kind === "detached" || sync.kind === "fetch") {
    return null;
  }
  if (sync.kind === "diverged") {
    return (
      <span className="truncate type-caption text-muted-foreground">
        {sync.behind} to pull and {sync.ahead} to push, merge in a terminal
      </span>
    );
  }

  const { action, label, busy } = describe(sync);

  return (
    <Button
      size="sm"
      variant={primary ? "default" : "ghost"}
      disabled={write.isPending}
      aria-busy={write.isPending}
      onClick={() => {
        write.mutate((client) =>
          client.call(
            action === "push" ? "checkouts.push" : "checkouts.pull",
            checkout.path,
          ),
        );
      }}
    >
      {write.isPending && <Spinner aria-hidden data-icon="inline-start" />}
      {write.isPending ? busy : label}
    </Button>
  );
}

function describe(sync: Extract<Sync, { kind: "publish" | "push" | "pull" }>) {
  switch (sync.kind) {
    case "publish":
      return {
        action: "push" as Action,
        label: "Publish branch",
        busy: "Publishing…",
      };
    case "push":
      return {
        action: "push" as Action,
        label: `Push ${String(sync.ahead)}`,
        busy: "Pushing…",
      };
    case "pull":
      return {
        action: "pull" as Action,
        label: `Pull ${String(sync.behind)}`,
        busy: "Pulling…",
      };
  }
}

const failures: Record<Sync["kind"], string> = {
  detached: "Could not sync",
  diverged: "Could not sync",
  publish: "Could not publish the branch",
  pull: "Could not pull",
  push: "Could not push",
  fetch: "Could not fetch",
};
