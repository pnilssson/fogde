import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";

import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { callOwn } from "@/lib/host";
import { report } from "@/lib/report";

export function useFinishOnboarding() {
  const queries = useQueryClient();
  const navigate = useNavigate();
  const finish = useMutation({
    mutationFn: () => callOwn("onboarding.finish"),
    onSuccess: () => {
      queries.setQueryData(onboardedQuery.queryKey, true);
      return navigate({ to: "/" });
    },
    onError: (error) => {
      report("Could not finish setting up", error);
    },
  });

  return {
    finish: () => {
      finish.mutate();
    },
    finishing: finish.isPending,
  };
}
