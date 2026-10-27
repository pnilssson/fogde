import { useQuery } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import type { Shown } from "@/generated/protocol";
import { agents, allAgents } from "@/lib/agents";
import { callOwn } from "@/lib/host";
import { shownQuery, shows } from "@/lib/usage";

export function UsageSettings() {
  const { data } = useQuery(shownQuery);
  const save = useSaveSetting(
    shownQuery,
    (next: Shown) => callOwn("usage.setShown", next),
    "Could not save the usage limit settings",
  );

  return (
    <SettingsSection
      id="usage-heading"
      title="Usage limits"
      hint="Shows how much of each plan is left in the status bar."
    >
      {allAgents.map((agent) => (
        <SettingsRow key={agent} label={agents[agent].short} labelled>
          <Switch
            checked={data !== undefined && shows(data, agent)}
            disabled={!data}
            onCheckedChange={(checked) => {
              if (data) {
                save.mutate({ ...data, [agent]: checked });
              }
            }}
          />
        </SettingsRow>
      ))}
    </SettingsSection>
  );
}
