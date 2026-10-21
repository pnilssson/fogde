import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import { toast } from "@/components/ui/toast";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import type { Notifications } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { notificationsQuery } from "@/lib/notifications";

export function NotificationSettings() {
  const queries = useQueryClient();
  const { data } = useQuery(notificationsQuery);
  const save = useMutation({
    mutationFn: (next: Notifications) => callOwn("notifications.set", next),
    onMutate: (next) => {
      const before = queries.getQueryData(notificationsQuery.queryKey);
      queries.setQueryData(notificationsQuery.queryKey, next);

      return before;
    },
    onError: (error, _, before) => {
      queries.setQueryData(notificationsQuery.queryKey, before);
      toast.add({
        type: "error",
        title: "Could not save the notification settings",
        description: error.message,
      });
    },
  });

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
