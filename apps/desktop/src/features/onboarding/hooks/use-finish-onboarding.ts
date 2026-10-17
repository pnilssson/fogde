import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";

import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { call } from "@/lib/host";

export function useFinishOnboarding() {
  const queries = useQueryClient();
  const navigate = useNavigate();
  const finish = useMutation({
    mutationFn: () => call("onboarding.finish"),
    onSuccess: () => {
      queries.setQueryData(onboardedQuery.queryKey, true);
      void navigate({ to: "/" });
    },
    throwOnError: true,
  });

  return () => {
    finish.mutate();
  };
}
