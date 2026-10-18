import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { useFinishOnboarding } from "@/features/onboarding/hooks/use-finish-onboarding";
import { remoteReposQuery } from "@/features/repos/api/repos";
import { CloneRepos } from "@/features/repos/components/clone-repos";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/clone")({
  pendingMs: stepPendingMs,
  loader: async ({ context: { queries } }) => {
    const accounts = await queries.query({
      ...accountsQuery,
      staleTime: "static",
    });
    const hosts = accounts.map((account) => account.host);

    await Promise.allSettled(
      hosts.map((host) =>
        queries.query({ ...remoteReposQuery(host), staleTime: "static" }),
      ),
    );

    return { hosts };
  },
  component: function CloneRoute() {
    const { hosts } = Route.useLoaderData();
    const navigate = useNavigate();
    const { finish, finishing } = useFinishOnboarding();

    return (
      <CloneRepos
        hosts={hosts}
        onBack={() => void navigate({ to: "/onboarding/repos" })}
        finishing={finishing}
        onContinue={finish}
      />
    );
  },
});
