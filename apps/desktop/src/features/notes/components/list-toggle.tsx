import { SidebarSimpleIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { toggleList, useListShown } from "@/features/notes/lib/list";

export function ListToggle() {
  const shown = useListShown();
  const label = shown ? "Hide notes list" : "Show notes list";

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label={label}
            onClick={toggleList}
          />
        }
      >
        <SidebarSimpleIcon weight={shown ? "fill" : "regular"} />
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}
