import { useQuery } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import type { Accessibility } from "@/generated/protocol";
import { accessibilityQuery } from "@/lib/accessibility";
import { callOwn } from "@/lib/host";

export function AccessibilitySettings() {
  const { data } = useQuery(accessibilityQuery);
  const save = useSaveSetting(
    accessibilityQuery,
    (next: Accessibility) => callOwn("accessibility.set", next),
    "Could not save the accessibility settings",
  );

  return (
    <SettingsSection id="accessibility-heading" title="Accessibility">
      <SettingsRow label="Screen reader" labelled>
        <Switch
          checked={data?.screenReader ?? false}
          disabled={!data}
          onCheckedChange={(screenReader) => {
            save.mutate({ screenReader });
          }}
        />
      </SettingsRow>
    </SettingsSection>
  );
}
