import { useQuery } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import { MonitorRows } from "@/features/settings/components/monitor-rows";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import type { Shown } from "@/generated/protocol";
import { agents, allAgents } from "@/lib/agents";
import { callOwn } from "@/lib/host";
import { shownQuery, shows } from "@/lib/usage";

export function StatusBarSettings() {
  const { data } = useQuery(shownQuery);
  const save = useSaveSetting(
    shownQuery,
    (next: Shown) => callOwn("usage.setShown", next),
    "Could not save the status bar settings",
  );

  return (
    <SettingsSection id="status-bar-heading" title="Status bar">
      {allAgents.map((agent) => (
        <SettingsRow
          key={agent}
          label={`${agents[agent].short} usage left`}
          labelled
        >
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
      <MonitorRows />
    </SettingsSection>
  );
}
