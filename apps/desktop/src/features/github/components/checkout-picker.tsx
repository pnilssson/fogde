import { CaretDownIcon, CheckIcon } from "@phosphor-icons/react";

import { Labelled } from "@/components/labelled";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { type Entry, pullFor } from "@/features/github/lib/scope";
import type { CheckoutStatus, Workspace } from "@/generated/protocol";
import { folderName } from "@/lib/folders";

export interface Group {
  workspace: Workspace;
  checkouts: CheckoutStatus[];
}

interface Props {
  groups: Group[];
  chosen: CheckoutStatus | undefined;
  openPulls: Entry[];
  onPick: (path: string) => void;
}

export function CheckoutPicker({ groups, chosen, openPulls, onPick }: Props) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button
            size="sm"
            variant="ghost"
            className="min-w-0"
            disabled={groups.length === 0}
          />
        }
      >
        <Labelled
          label="Checkout"
          value={chosen ? folderName(chosen.path) : "No checkouts"}
        />
        <CaretDownIcon data-icon="inline-end" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="max-h-96 w-auto min-w-72">
        {groups.map(({ workspace, checkouts }) => (
          <DropdownMenuGroup key={workspace.id}>
            <DropdownMenuLabel>{workspace.name}</DropdownMenuLabel>
            {checkouts.map((checkout) => (
              <DropdownMenuItem
                key={checkout.path}
                className="items-start"
                onClick={() => {
                  onPick(checkout.path);
                }}
              >
                <span className="flex size-4 shrink-0">
                  {checkout.path === chosen?.path && (
                    <CheckIcon aria-label="Shown" />
                  )}
                </span>
                <span className="flex min-w-0 flex-1 flex-col">
                  <span className="truncate">{folderName(checkout.path)}</span>
                  <span className="truncate type-caption text-muted-foreground">
                    {describe(
                      checkout,
                      pullFor(openPulls, checkout)?.item.number,
                    )}
                  </span>
                </span>
              </DropdownMenuItem>
            ))}
          </DropdownMenuGroup>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function describe(checkout: CheckoutStatus, pull: number | undefined) {
  const changed = checkout.files.length;
  const parts = [
    checkout.repo
      ? `${checkout.branch ?? "Detached"} in ${checkout.repo.owner}/${checkout.repo.name}`
      : (checkout.branch ?? "Detached"),
    changed > 0 &&
      `${checkout.truncated ? `${String(changed)}+` : String(changed)} changed`,
    pull !== undefined && `pull request #${String(pull)}`,
  ];

  return parts.filter(Boolean).join(", ");
}
