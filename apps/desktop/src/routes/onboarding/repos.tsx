import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { remoteReposQuery, reposFolderQuery } from "@/features/repos/api/repos";
import { ChooseFolder } from "@/features/repos/components/choose-folder";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/repos")({
  pendingMs: stepPendingMs,
  loader: ({ context: { queries } }) => {
    void queries
      .query({ ...accountsQuery, staleTime: "static" })
      .then((accounts) =>
        Promise.allSettled(
          accounts.map((account) =>
            queries.query({
              ...remoteReposQuery(account.host),
              staleTime: "static",
            }),
          ),
        ),
      )
      .catch(() => undefined);

    return queries.query({ ...reposFolderQuery, staleTime: "static" });
  },
  component: function ReposRoute() {
    const navigate = useNavigate();

    return (
      <ChooseFolder
        onBack={() => void navigate({ to: "/onboarding/agents" })}
        onContinue={() => navigate({ to: "/onboarding/clone" })}
      />
    );
  },
});
