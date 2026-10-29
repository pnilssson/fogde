import { XIcon } from "@phosphor-icons/react";
import { type ReactNode, useContext } from "react";

import { Bar } from "@/components/frame";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { PageClose } from "@/lib/page-close";

export function PageBar({ children }: { children: ReactNode }) {
  const close = useContext(PageClose);

  return (
    <Bar>
      {children}
      {close && (
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Close"
                onClick={close}
              />
            }
          >
            <XIcon />
          </TooltipTrigger>
          <TooltipContent side="bottom">
            Close
            <Kbd>Esc</Kbd>
          </TooltipContent>
        </Tooltip>
      )}
    </Bar>
  );
}
