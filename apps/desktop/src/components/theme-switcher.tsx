import { DesktopIcon, MoonIcon, SunIcon } from "@phosphor-icons/react";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { chooseTheme, useTheme } from "@/lib/theme";

const themes = [
  { value: "system", label: "System", icon: DesktopIcon },
  { value: "light", label: "Light", icon: SunIcon },
  { value: "dark", label: "Dark", icon: MoonIcon },
] as const;

export function ThemeSwitcher() {
  const theme = useTheme();

  return (
    <ToggleGroup
      variant="outline"
      size="sm"
      spacing={0}
      aria-label="Theme"
      value={[theme]}
      onValueChange={(values) => {
        const next = themes.find(({ value }) => value === values[0]);
        if (next) {
          void chooseTheme(next.value);
        }
      }}
    >
      {themes.map(({ value, label, icon: Icon }) => (
        <Tooltip key={value}>
          <TooltipTrigger
            render={<ToggleGroupItem value={value} aria-label={label} />}
          >
            <Icon />
          </TooltipTrigger>
          <TooltipContent>{label}</TooltipContent>
        </Tooltip>
      ))}
    </ToggleGroup>
  );
}
