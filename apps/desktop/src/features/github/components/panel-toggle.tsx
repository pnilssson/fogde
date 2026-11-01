import { SidebarSimpleIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { togglePanel, usePanel } from "@/features/github/lib/panel";

export function PanelToggle() {
  const open = usePanel((panel) => panel.open);
  const label = open ? "Hide changes" : "Show changes";

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={label}
            aria-pressed={open}
            onClick={togglePanel}
          />
        }
      >
        <SidebarSimpleIcon
          className="-scale-x-100"
          weight={open ? "fill" : "regular"}
        />
      </TooltipTrigger>
      <TooltipContent side="bottom">{label}</TooltipContent>
    </Tooltip>
  );
}
