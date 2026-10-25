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
  hint?: string;
  action?: ReactNode;
  children: ReactNode;
}

export function SettingsSection({
  id,
  title,
  hint,
  action,
  children,
}: SectionProps) {
  return (
    <section className="flex flex-col gap-1" aria-labelledby={id}>
      <div className="flex h-8 items-center gap-2">
        <div className="flex min-w-0 flex-1 items-center gap-1">
          <h2 id={id} className="min-w-0 truncate type-title">
            {title}
          </h2>
          {hint && <Hint text={hint} />}
        </div>
        {action}
      </div>
      {children}
    </section>
  );
}

interface RowProps {
  label: ReactNode;
  leading?: ReactNode;
  labelled?: boolean;
  children: ReactNode;
}

export function SettingsRow({
  label,
  leading,
  labelled = false,
  children,
}: RowProps) {
  const Root = labelled ? Field.Root : "div";
  const Label = labelled ? Field.Label : "span";

  return (
    <Root className="flex h-8 items-center gap-2">
      {leading}
      <Label className="min-w-0 flex-1 truncate type-body">{label}</Label>
      {children}
    </Root>
  );
}

function Hint({ text }: { text: string }) {
  return (
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
      <TooltipContent>{text}</TooltipContent>
    </Tooltip>
  );
}
