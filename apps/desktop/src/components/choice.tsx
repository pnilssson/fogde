import type { Icon } from "@phosphor-icons/react";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface Option<T extends string> {
  value: T;
  label: string;
  icon?: Icon;
}

interface Props<T extends string> {
  label: string;
  value: T;
  options: readonly Option<T>[];
  onChange: (value: T) => void;
}

const item =
  "h-8 px-2 font-normal type-body text-muted-foreground hover:bg-hover hover:text-foreground aria-pressed:bg-surface-3 aria-pressed:text-foreground aria-pressed:shadow-surface-2";

export function Choice<T extends string>({
  label,
  value,
  options,
  onChange,
}: Props<T>) {
  return (
    <ToggleGroup
      size="sm"
      spacing={2}
      aria-label={label}
      value={[value]}
      onValueChange={(values) => {
        const next = options.find((option) => option.value === values[0]);
        if (next) {
          onChange(next.value);
        }
      }}
    >
      {options.map(({ value: option, label: name, icon: Glyph }) =>
        Glyph ? (
          <Tooltip key={option}>
            <TooltipTrigger
              render={
                <ToggleGroupItem
                  value={option}
                  aria-label={name}
                  className={item}
                />
              }
            >
              <Glyph />
            </TooltipTrigger>
            <TooltipContent>{name}</TooltipContent>
          </Tooltip>
        ) : (
          <ToggleGroupItem key={option} value={option} className={item}>
            {name}
          </ToggleGroupItem>
        ),
      )}
    </ToggleGroup>
  );
}
