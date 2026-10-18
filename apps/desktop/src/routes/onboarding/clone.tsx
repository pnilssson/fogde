import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { useFinishOnboarding } from "@/features/onboarding/hooks/use-finish-onboarding";
import { remoteReposQuery } from "@/features/repos/api/repos";
import { CloneRepos } from "@/features/repos/components/clone-repos";

export const Route = createFileRoute("/onboarding/clone")({
  loader: async ({ context: { queries } }) => {
    const accounts = await queries.query({
      ...accountsQuery,
      staleTime: "static",
    });
    const hosts = accounts.map((account) => account.host);

    // A host that fails shows a toast on the page instead of blocking it.
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
    const finish = useFinishOnboarding();

    return (
      <CloneRepos
        hosts={hosts}
        onBack={() => void navigate({ to: "/onboarding/repos" })}
        onContinue={finish}
      />
    );
  },
});
