import { useState } from "react";

import { ConfirmDialog } from "@/components/confirm-dialog";
import { Button } from "@/components/ui/button";
import { StateIcon } from "@/features/github/components/rows";
import { useWrite } from "@/features/github/hooks/use-write";
import type { Entry } from "@/features/github/lib/scope";
import type { CheckoutStatus } from "@/generated/protocol";
import { folderName } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  checkout: CheckoutStatus;
  entry: Entry;
  linked: boolean;
}

export function LandedCard({ machine, checkout, entry, linked }: Props) {
  const [open, setOpen] = useState(false);
  const clean = useWrite(machine, "Could not clean up");
  const { item } = entry;
  const base = item.pull?.base ?? "";
  const sha = item.pull?.sha ?? "";
  const branch = checkout.branch ?? "the branch";

  return (
    <div className="flex items-center gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
      <span className="flex shrink-0">
        <StateIcon kind="pull" state="merged" draft={false} />
      </span>
      <p className="min-w-0 flex-1 type-body">
        {item.title}{" "}
        <span className="text-muted-foreground">#{item.number}</span> was merged
        into {base}.
      </p>
      <Button
        size="sm"
        onClick={() => {
          setOpen(true);
        }}
      >
        Clean up
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={`Clean up after #${String(item.number)}?`}
        description={
          linked
            ? `Removes the worktree ${folderName(checkout.path)}, closes its terminals and deletes ${branch}.`
            : `Switches to ${base}, pulls it and deletes ${branch}.`
        }
        action="Clean up"
        busy="Cleaning up…"
        pending={clean.isPending}
        safe
        onConfirm={() => {
          clean.mutate(
            (client) =>
              client.call("checkouts.cleanUp", {
                path: checkout.path,
                sha,
                base,
              }),
            {
              onSettled: () => {
                setOpen(false);
              },
            },
          );
        }}
      />
    </div>
  );
}
