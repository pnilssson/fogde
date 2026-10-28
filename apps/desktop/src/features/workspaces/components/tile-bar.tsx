import { XIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { type Dragged, dragProps } from "@/features/workspaces/lib/drag";

interface Props {
  item: Dragged | undefined;
  active: boolean;
  icon: ReactNode;
  title: string;
  model?: string | null | undefined;
  onClose?: (() => void) | undefined;
}

export function TileBar({ item, active, icon, title, model, onClose }: Props) {
  return (
    <header
      {...(item ? dragProps(item) : {})}
      className={cn(
        "@container flex h-8 shrink-0 items-center gap-2 px-2 font-mono type-caption",
        item && "cursor-grab active:cursor-grabbing",
        !active && "text-muted-foreground",
      )}
    >
      {icon}
      <span className="truncate">{title}</span>
      <span className="flex-1" />
      {model && (
        <span className="hidden shrink-0 text-muted-foreground @lg:inline">
          {model}
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
                onClick={() => {
                  onClose();
                }}
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
