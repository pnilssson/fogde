import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { Switch } from "@/components/ui/switch";
import { toast } from "@/components/ui/toast";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import type { Accessibility } from "@/generated/protocol";
import { accessibilityQuery } from "@/lib/accessibility";
import { callOwn } from "@/lib/host";

export function AccessibilitySettings() {
  const queries = useQueryClient();
  const { data } = useQuery(accessibilityQuery);
  const save = useMutation({
    mutationFn: (next: Accessibility) => callOwn("accessibility.set", next),
    onMutate: (next) => {
      const before = queries.getQueryData(accessibilityQuery.queryKey);
      queries.setQueryData(accessibilityQuery.queryKey, next);

      return before;
    },
    onError: (error, _, before) => {
      queries.setQueryData(accessibilityQuery.queryKey, before);
      toast.add({
        type: "error",
        title: "Could not save the accessibility settings",
        description: error.message,
      });
    },
  });

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
