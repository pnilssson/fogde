import { XIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import type { ReactNode } from "react";

import { BarTitle } from "@/components/bar-title";
import { Bar } from "@/components/frame";
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
    <Bar
      {...(item ? dragProps(item) : {})}
      className={cn(
        "@container font-mono type-caption",
        item && "cursor-grab active:cursor-grabbing",
        !active && "text-muted-foreground",
      )}
    >
      <BarTitle icon={icon} title={title} />
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
    </Bar>
  );
}
