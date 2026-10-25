import { useQuery } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import type { Shown } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { shownQuery } from "@/lib/usage";

const rows: { key: keyof Shown; label: string }[] = [
  { key: "claude", label: "Claude usage left" },
  { key: "codex", label: "Codex usage left" },
  { key: "copilot", label: "GitHub Copilot usage left" },
];

export function UsageSettings() {
  const { data } = useQuery(shownQuery);
  const save = useSaveSetting(
    shownQuery,
    (next: Shown) => callOwn("usage.setShown", next),
    "Could not save the status bar settings",
  );

  return (
    <SettingsSection id="status-bar-heading" title="Status bar">
      {rows.map(({ key, label }) => (
        <SettingsRow key={key} label={label} labelled>
          <Switch
            checked={data?.[key] ?? false}
            disabled={!data}
            onCheckedChange={(checked) => {
              if (data) {
                save.mutate({ ...data, [key]: checked });
              }
            }}
          />
        </SettingsRow>
      ))}
    </SettingsSection>
  );
}
