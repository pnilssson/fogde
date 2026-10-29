import { CaretDownIcon, GitBranchIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Spinner } from "@/components/ui/spinner";
import { useWrite } from "@/features/github/hooks/use-write";
import { holds } from "@/features/github/lib/scope";
import { select } from "@/features/github/lib/state";
import type { CheckoutStatus, PullDetail, Repo } from "@/generated/protocol";
import { folderName } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  repo: Repo;
  number: number;
  pull: PullDetail;
  checkouts: CheckoutStatus[];
}

export function CheckoutMenu({
  machine,
  repo,
  number,
  pull,
  checkouts,
}: Props) {
  const write = useWrite<string>(
    machine,
    "Could not check out the pull request",
  );
  const holder = checkouts.find((checkout) => holds(checkout, number, pull));
  const first = checkouts[0];

  if (holder) {
    return (
      <Button
        size="sm"
        variant="secondary"
        onClick={() => {
          select(machine.id, { kind: "checkout", path: holder.path });
        }}
      >
        <GitBranchIcon data-icon="inline-start" />
        Open in {folderName(holder.path)}
      </Button>
    );
  }
  if (!first) {
    return null;
  }

  const checkOut = (path: string, worktree: boolean) => {
    write.mutate(
      (client) =>
        client.call("github.checkoutPull", { path, repo, number, worktree }),
      {
        onSuccess: (checkout) => {
          select(machine.id, { kind: "checkout", path: checkout });
        },
      },
    );
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button
            size="sm"
            variant="secondary"
            disabled={write.isPending}
            aria-busy={write.isPending}
          />
        }
      >
        {write.isPending ? (
          <Spinner aria-hidden data-icon="inline-start" />
        ) : (
          <GitBranchIcon data-icon="inline-start" />
        )}
        {write.isPending ? "Checking out…" : "Check out"}
        <CaretDownIcon data-icon="inline-end" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-auto min-w-56">
        <DropdownMenuItem
          onClick={() => {
            checkOut(first.path, true);
          }}
        >
          In a new worktree
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        {checkouts.map((checkout) => (
          <DropdownMenuItem
            key={checkout.path}
            onClick={() => {
              checkOut(checkout.path, false);
            }}
          >
            <span className="min-w-0 flex-1 truncate">
              In {folderName(checkout.path)}
            </span>
            <span className="shrink-0 type-caption text-muted-foreground">
              replaces {checkout.branch ?? "a detached head"}
            </span>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
