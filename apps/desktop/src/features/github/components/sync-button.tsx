import { Labelled } from "@/components/labelled";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useWrite } from "@/features/github/hooks/use-write";
import { ago } from "@/features/github/lib/scope";
import type { Sync } from "@/features/github/lib/sync";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  checkout: CheckoutStatus;
  sync: Sync;
  primary: boolean;
}

type Action = "fetch" | "push" | "pull";

export function SyncButton({ machine, checkout, sync, primary }: Props) {
  const write = useWrite(machine, failures[sync.kind]);
  if (sync.kind === "detached") {
    return null;
  }

  const { action, label, busy, note } = describe(sync, checkout);
  const moving = action !== "fetch";

  return (
    <Button
      size="sm"
      variant={primary && moving ? "default" : "ghost"}
      disabled={write.isPending}
      aria-busy={write.isPending}
      onClick={() => {
        write.mutate((client) => {
          switch (action) {
            case "fetch":
              return client.call("checkouts.fetch", checkout.path);
            case "push":
              return client.call("checkouts.push", checkout.path);
            case "pull":
              return client.call("checkouts.pull", checkout.path);
          }
        });
      }}
    >
      {write.isPending && <Spinner aria-hidden data-icon="inline-start" />}
      <Labelled value={write.isPending ? busy : label} label={note} under />
    </Button>
  );
}

function describe(
  sync: Exclude<Sync, { kind: "detached" }>,
  checkout: CheckoutStatus,
) {
  const upstream = checkout.upstream ?? "the remote";
  const fetched =
    checkout.fetched === null
      ? "Never fetched"
      : `Fetched ${ago(checkout.fetched)}`;

  switch (sync.kind) {
    case "publish":
      return {
        action: "push" as Action,
        label: "Publish branch",
        busy: "Publishing…",
        note: "Not on the remote yet",
      };
    case "push":
      return {
        action: "push" as Action,
        label: `Push ${String(sync.ahead)}`,
        busy: "Pushing…",
        note: `To ${upstream}`,
      };
    case "pull":
      return {
        action: "pull" as Action,
        label: `Pull ${String(sync.behind)}`,
        busy: "Pulling…",
        note: `From ${upstream}`,
      };
    case "diverged":
      return {
        action: "fetch" as Action,
        label: "Fetch",
        busy: "Fetching…",
        note: `${String(sync.behind)} to pull and ${String(sync.ahead)} to push, merge in a terminal`,
      };
    case "fetch":
      return {
        action: "fetch" as Action,
        label: "Fetch",
        busy: "Fetching…",
        note: fetched,
      };
  }
}

const failures: Record<Sync["kind"], string> = {
  detached: "Could not sync",
  diverged: "Could not fetch",
  publish: "Could not publish the branch",
  pull: "Could not pull",
  push: "Could not push",
  fetch: "Could not fetch",
};
