import { GitBranchIcon, XIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useActiveClose } from "@/features/workspaces/lib/closing";
import { type Dragged, dragProps } from "@/features/workspaces/lib/drag";

interface Props {
  item: Dragged | undefined;
  active: boolean;
  icon: ReactNode;
  title: string;
  detail?: string | undefined;
  branch?: string | null | undefined;
  onClose?: (() => void) | undefined;
}

export function TileBar({
  item,
  active,
  icon,
  title,
  detail,
  branch,
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
      <span className="sr-only">{title}</span>
      <span className="min-w-0 flex-1 truncate">{detail}</span>
      {branch && (
        <span className="flex max-w-1/3 min-w-0 shrink-0 items-center gap-2">
          <GitBranchIcon aria-label="Branch" className="size-4 shrink-0" />
          <span className="truncate">{branch}</span>
        </span>
      )}
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
