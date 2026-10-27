import { ThemeSwitcher } from "@/components/theme-switcher";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { TranslucentSwitch } from "@/features/settings/components/translucent-switch";
import { frame } from "@/lib/frame";

export function AppearanceSettings() {
  return (
    <SettingsSection id="appearance-heading" title="Appearance">
      <SettingsRow label="Theme">
        <ThemeSwitcher />
      </SettingsRow>
      {frame.translucent && (
        <SettingsRow label="Translucent" labelled>
          <TranslucentSwitch />
        </SettingsRow>
      )}
    </SettingsSection>
  );
}
