import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { ConnectGitHub } from "@/features/github/components/connect-github";
import { serverQuery } from "@/features/server/api/server";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/github")({
  pendingMs: stepPendingMs,
  loader: ({ context: { queries } }) =>
    Promise.all([
      queries.query({ ...accountsQuery, staleTime: "static" }),
      queries.query({ ...serverQuery, staleTime: "static" }),
    ]),
  component: function GitHubRoute() {
    const navigate = useNavigate();
    const { data: server } = useSuspenseQuery(serverQuery);

    return (
      <ConnectGitHub
        onBack={() =>
          void navigate({
            to: server === null ? "/onboarding/where" : "/onboarding/server",
          })
        }
        onContinue={() => navigate({ to: "/onboarding/agents" })}
      />
    );
  },
});
