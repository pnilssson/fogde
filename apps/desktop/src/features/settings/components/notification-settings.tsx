import { useQuery } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useSaveSetting } from "@/features/settings/hooks/use-save-setting";
import type { Notifications } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { notificationsQuery } from "@/lib/notifications";

export function NotificationSettings() {
  const { data } = useQuery(notificationsQuery);
  const save = useSaveSetting(
    notificationsQuery,
    (next: Notifications) => callOwn("notifications.set", next),
    "Could not save the notification settings",
  );

  return (
    <SettingsSection id="alerts-heading" title="Alerts">
      <SettingsRow label="Notifications" labelled>
        <Switch
          checked={data?.enabled ?? false}
          disabled={!data}
          onCheckedChange={(enabled) => {
            if (data) {
              save.mutate({ ...data, enabled });
            }
          }}
        />
      </SettingsRow>
      <SettingsRow label="Sound" labelled>
        <Switch
          checked={data?.sound ?? false}
          disabled={!data}
          onCheckedChange={(sound) => {
            if (data) {
              save.mutate({ ...data, sound });
            }
          }}
        />
      </SettingsRow>
    </SettingsSection>
  );
}
