import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";

import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { callOwn } from "@/lib/host";

export function useFinishOnboarding() {
  const queries = useQueryClient();
  const navigate = useNavigate();
  const finish = useMutation({
    mutationFn: () => callOwn("onboarding.finish"),
    onSuccess: () => {
      queries.setQueryData(onboardedQuery.queryKey, true);
      return navigate({ to: "/" });
    },
    throwOnError: true,
  });

  return {
    finish: () => {
      finish.mutate();
    },
    finishing: finish.isPending,
  };
}
