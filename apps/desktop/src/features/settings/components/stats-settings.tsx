import { useQuery } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import type { Stats } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { statsQuery } from "@/lib/stats";

const rows: { key: keyof Stats; label: string }[] = [
  { key: "behind", label: "Commits behind the default branch" },
  { key: "lines", label: "Lines changed on the branch" },
  { key: "upstream", label: "Commits to push and pull" },
];

export function StatsSettings() {
  const { data } = useQuery(statsQuery);
  const save = useSaveSetting(
    statsQuery,
    (next: Stats) => callOwn("stats.set", next),
    "Could not save the branch settings",
  );

  return (
    <SettingsSection
      id="branch-heading"
      title="Branches"
      hint="Shows how each branch differs beside it in the sidebar."
    >
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
