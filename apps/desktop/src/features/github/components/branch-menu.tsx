import { CaretDownIcon, CheckIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Labelled } from "@/components/labelled";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Spinner } from "@/components/ui/spinner";
import { branchesQuery } from "@/features/github/api/github";
import { useWrite } from "@/features/github/hooks/use-write";
import { ago, samePath } from "@/features/github/lib/scope";
import type { Branch } from "@/generated/protocol";
import { folderName } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  path: string;
  branch: string | null;
  onOpen: (path: string) => void;
}

export function BranchMenu({ machine, path, branch, onOpen }: Props) {
  const [open, setOpen] = useState(false);
  const branches = useQuery({ ...branchesQuery(machine, path), enabled: open });
  const change = useWrite(machine, "Could not switch branch");

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger
        render={
          <Button
            size="sm"
            variant="ghost"
            className="min-w-0"
            disabled={change.isPending}
            aria-busy={change.isPending}
          />
        }
      >
        <Labelled
          label="Branch"
          value={change.isPending ? "Switching…" : (branch ?? "Detached")}
        />
        {change.isPending ? (
          <Spinner aria-hidden data-icon="inline-end" />
        ) : (
          <CaretDownIcon data-icon="inline-end" />
        )}
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="max-h-96 w-auto min-w-64">
        {branches.isPending && (
          <div className="flex items-center gap-2 p-2">
            <Spinner aria-hidden />
            <span className="type-caption">Loading branches…</span>
          </div>
        )}
        {branches.data?.map((item) => (
          <BranchItem
            key={item.name}
            item={item}
            current={item.name === branch}
            here={path}
            onSwitch={() => {
              change.mutate((client) =>
                client.call("checkouts.switch", { path, branch: item.name }),
              );
            }}
            onOpen={onOpen}
          />
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

interface ItemProps {
  item: Branch;
  current: boolean;
  here: string;
  onSwitch: () => void;
  onOpen: (path: string) => void;
}

function BranchItem({ item, current, here, onSwitch, onOpen }: ItemProps) {
  const elsewhere =
    item.worktree && !samePath(item.worktree, here) ? item.worktree : null;
  const detail = elsewhere
    ? `In worktree ${folderName(elsewhere)}`
    : item.remote
      ? `Remote, ${ago(item.date)}`
      : ago(item.date);

  return (
    <DropdownMenuItem
      disabled={current}
      onClick={() => {
        if (elsewhere) {
          onOpen(elsewhere);
          return;
        }
        onSwitch();
      }}
    >
      <span className="flex size-4 shrink-0">
        {current && <CheckIcon aria-label="Current branch" />}
      </span>
      <span className="min-w-0 flex-1 truncate">{item.name}</span>
      <span className="shrink-0 type-caption text-muted-foreground">
        {detail}
      </span>
    </DropdownMenuItem>
  );
}
