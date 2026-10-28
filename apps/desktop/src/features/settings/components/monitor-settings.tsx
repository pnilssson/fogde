import { useQuery } from "@tanstack/react-query";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import { useMachines } from "@/hooks/use-machines";
import type { Bar } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { idOf } from "@/lib/machines";
import { barMachine, barQuery, metrics } from "@/lib/monitor";

export function MonitorSettings() {
  const { data } = useQuery(barQuery);
  const { all } = useMachines();
  const save = useSaveSetting(
    barQuery,
    (next: Bar) => callOwn("monitor.setBar", next),
    "Could not save the monitoring settings",
  );
  const chosen = data && barMachine(data, all);

  return (
    <SettingsSection
      id="monitor-heading"
      title="Monitoring"
      hint="Shows what Fogde uses on one machine beside Monitor in the sidebar."
    >
      <SettingsRow label="Machine" labelled>
        <Select
          items={all.map(({ id, name }) => ({ label: name, value: id }))}
          value={chosen?.id ?? null}
          disabled={!data}
          onValueChange={(id) => {
            const machine = all.find((candidate) => candidate.id === id);
            if (data && machine) {
              save.mutate({ ...data, machine: idOf(machine) });
            }
          }}
        >
          <SelectTrigger size="sm" aria-label="Machine to monitor">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {all.map(({ id, name }) => (
              <SelectItem key={id} value={id}>
                {name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </SettingsRow>
      {metrics.map(({ metric, label }) => (
        <SettingsRow key={metric} label={label} labelled>
          <Switch
            checked={data?.metrics.includes(metric) ?? false}
            disabled={!data}
            onCheckedChange={(checked) => {
              if (!data) {
                return;
              }

              save.mutate({
                ...data,
                metrics: metrics
                  .map((entry) => entry.metric)
                  .filter((each) =>
                    each === metric ? checked : data.metrics.includes(each),
                  ),
              });
            }}
          />
        </SettingsRow>
      ))}
    </SettingsSection>
  );
}
