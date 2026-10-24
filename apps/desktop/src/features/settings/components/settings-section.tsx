import { Field } from "@base-ui/react/field";
import { InfoIcon } from "@phosphor-icons/react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface SectionProps {
  id: string;
  title: string;
  action?: ReactNode;
  children: ReactNode;
}

export function SettingsSection({ id, title, action, children }: SectionProps) {
  return (
    <section className="flex flex-col gap-1" aria-labelledby={id}>
      <div className="flex h-8 items-center gap-2">
        <h2 id={id} className="min-w-0 flex-1 truncate type-title">
          {title}
        </h2>
        {action}
      </div>
      {children}
    </section>
  );
}

interface RowProps {
  label: ReactNode;
  leading?: ReactNode;
  hint?: string;
  labelled?: boolean;
  children: ReactNode;
}

export function SettingsRow({
  label,
  leading,
  hint,
  labelled = false,
  children,
}: RowProps) {
  const Root = labelled ? Field.Root : "div";
  const Label = labelled ? Field.Label : "span";

  return (
    <Root className="flex h-8 items-center gap-2">
      {leading}
      <div className="flex min-w-0 flex-1 items-center gap-1">
        <Label className="min-w-0 truncate type-body">{label}</Label>
        {hint && (
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-xs"
                  aria-label="More information"
                />
              }
            >
              <InfoIcon />
            </TooltipTrigger>
            <TooltipContent>{hint}</TooltipContent>
          </Tooltip>
        )}
      </div>
      {children}
    </Root>
  );
}
