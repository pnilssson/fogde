import { GitBranchIcon } from "@phosphor-icons/react";
import { useId } from "react";

import { Spinner } from "@/components/ui/spinner";
import { useSpaces } from "@/features/monitor/hooks/use-spaces";
import { bytes } from "@/features/monitor/lib/format";
import type { Disk, Workspace } from "@/generated/protocol";
import { folderName } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspaces: Workspace[];
  disk: Disk | null;
}

export function Storage({ machine, workspaces, disk }: Props) {
  const heading = useId();
  const checkouts = workspaces.flatMap((workspace) =>
    workspace.checkouts.map((checkout) => ({
      ...checkout,
      workspace: workspace.name,
    })),
  );
  const spaces = useSpaces(
    machine,
    checkouts.map(({ path }) => path),
  );
  if (!disk && checkouts.length === 0) {
    return null;
  }

  const rows = checkouts
    .flatMap((checkout) => {
      const space = spaces.data?.find(({ path }) => path === checkout.path);

      return space ? [{ ...checkout, bytes: space.bytes }] : [];
    })
    .sort((first, second) => second.bytes - first.bytes);

  return (
    <section className="flex flex-col gap-2" aria-labelledby={heading}>
      <div className="flex items-baseline gap-2">
        <h2 id={heading} className="type-title">
          Storage
        </h2>
        {disk && (
          <span className="ml-auto font-mono type-caption text-muted-foreground tabular-nums">
            {bytes(disk.free)} free of {bytes(disk.total)}
          </span>
        )}
      </div>
      <ul className="flex flex-col type-body">
        {rows.map((row) => (
          <li key={row.path} className="flex h-8 items-center gap-2">
            <GitBranchIcon className="size-4 shrink-0" />
            <span className="truncate">
              {row.branch ?? folderName(row.path)}
            </span>
            <span className="truncate text-muted-foreground">
              in {row.workspace}
            </span>
            <span className="ml-auto font-mono tabular-nums">
              {bytes(row.bytes)}
            </span>
          </li>
        ))}
      </ul>
      {spaces.isPending && checkouts.length > 0 && <Spinner />}
    </section>
  );
}
