import { SidebarSimpleIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import { useSidebar } from "@/components/ui/sidebar";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export function SidebarToggle() {
  const { isMobile, open, openMobile, toggleSidebar } = useSidebar();
  const expanded = isMobile ? openMobile : open;
  const label = expanded ? "Hide sidebar" : "Show sidebar";

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="sm"
            aria-label={label}
            aria-expanded={expanded}
            onClick={toggleSidebar}
          />
        }
      >
        <SidebarSimpleIcon weight={expanded ? "fill" : "regular"} />
      </TooltipTrigger>
      <TooltipContent side="bottom">{label}</TooltipContent>
    </Tooltip>
  );
}
