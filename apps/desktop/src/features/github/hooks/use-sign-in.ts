import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { openUrl } from "@tauri-apps/plugin-opener";

import { userQuery } from "@/features/github/api/user";
import { call } from "@/lib/host";

export function useSignIn() {
  const queries = useQueryClient();
  const navigate = useNavigate();

  const finish = useMutation({
    mutationFn: () => call("github.finishSignIn"),
    onSuccess: (user) => {
      queries.setQueryData(userQuery.queryKey, user);
      void navigate({ to: "/" });
    },
  });
  const start = useMutation({
    mutationFn: () => call("github.startSignIn"),
    onSuccess: (prompt) => {
      void openUrl(prompt.url);
      finish.mutate();
    },
  });

  return {
    prompt: start.data,
    starting: start.isPending,
    error: start.error ?? finish.error,
    begin: () => {
      finish.reset();
      start.mutate();
    },
  };
}
