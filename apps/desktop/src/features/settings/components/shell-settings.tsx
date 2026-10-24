import { useQuery } from "@tanstack/react-query";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import type { Shell } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { shellQuery, shellsQuery } from "@/lib/shell";

const system = "";

export function ShellSettings() {
  const { data } = useQuery(shellQuery);
  const options = useQuery(shellsQuery).data ?? [];
  const save = useSaveSetting(
    shellQuery,
    (next: Shell) => callOwn("shell.set", next),
    "Could not save the shell",
  );
  const choices = [{ name: "System default", program: system }, ...options];

  return (
    <SettingsSection id="terminal-heading" title="Terminal">
      <SettingsRow label="Shell" labelled>
        <Select
          items={choices.map(({ name, program }) => ({
            label: name,
            value: program,
          }))}
          value={data?.program ?? system}
          disabled={!data}
          onValueChange={(program) => {
            save.mutate({ program: program === system ? null : program });
          }}
        >
          <SelectTrigger size="sm" aria-label="Shell">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {choices.map(({ name, program }) => (
              <SelectItem key={program} value={program}>
                {name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </SettingsRow>
    </SettingsSection>
  );
}
