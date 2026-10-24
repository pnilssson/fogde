import { GitBranchIcon, XIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { BranchStats } from "@/features/workspaces/components/branch-stats";
import { StateIcon } from "@/features/workspaces/components/state-icon";
import { useActiveClose } from "@/features/workspaces/lib/closing";
import { type Dragged, dragProps } from "@/features/workspaces/lib/drag";
import type { Changes, Status } from "@/generated/protocol";

interface Props {
  item: Dragged | undefined;
  active: boolean;
  icon: ReactNode;
  title: string;
  status?: Status | undefined;
  place?: Place | undefined;
  branch?: string | null | undefined;
  changes?: Changes | null | undefined;
  onClose?: (() => void) | undefined;
}

interface Place {
  label: string;
  path: string;
}

export function TileBar({
  item,
  active,
  icon,
  title,
  status,
  place,
  branch,
  changes,
  onClose,
}: Props) {
  useActiveClose(active, onClose);

  return (
    <header
      {...(item ? dragProps(item) : {})}
      className={cn(
        "flex h-8 shrink-0 items-center gap-2 border-b px-2 type-caption",
        item && "cursor-grab active:cursor-grabbing",
        !active && "text-muted-foreground",
      )}
    >
      {icon}
      <div className="flex min-w-0 flex-1 items-center gap-2">
        <span className="truncate">{title}</span>
        {status && <StateIcon status={status} />}
      </div>
      {place && (
        <Tooltip>
          <TooltipTrigger
            render={
              <span className="max-w-1/3 min-w-0 shrink-0 truncate text-muted-foreground" />
            }
          >
            {place.label}
          </TooltipTrigger>
          <TooltipContent>{place.path}</TooltipContent>
        </Tooltip>
      )}
      {branch && (
        <span className="flex max-w-1/3 min-w-0 shrink-0 items-center gap-2">
          <GitBranchIcon aria-label="Branch" className="size-4 shrink-0" />
          <span className="truncate">{branch}</span>
        </span>
      )}
      <BranchStats changes={changes} />
      {onClose && (
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-xs"
                aria-label={`Close ${title}`}
                onClick={onClose}
              />
            }
          >
            <XIcon />
          </TooltipTrigger>
          <TooltipContent>Close</TooltipContent>
        </Tooltip>
      )}
    </header>
  );
}
