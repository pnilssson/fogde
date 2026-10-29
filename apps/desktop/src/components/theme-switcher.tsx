import { DesktopIcon, MoonIcon, SunIcon } from "@phosphor-icons/react";

import { Choice } from "@/components/choice";
import { chooseTheme, useTheme } from "@/lib/theme";

const themes = [
  { value: "system", label: "System", icon: DesktopIcon },
  { value: "light", label: "Light", icon: SunIcon },
  { value: "dark", label: "Dark", icon: MoonIcon },
] as const;

export function ThemeSwitcher() {
  const theme = useTheme();

  return (
    <Choice
      label="Theme"
      value={theme}
      options={themes}
      onChange={(next) => {
        void chooseTheme(next);
      }}
    />
  );
}
