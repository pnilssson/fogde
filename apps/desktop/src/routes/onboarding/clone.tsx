import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { useFinishOnboarding } from "@/features/onboarding/hooks/use-finish-onboarding";
import {
  StepSearch,
  stepMachine,
} from "@/features/onboarding/lib/step-machine";
import { remoteReposQuery } from "@/features/repos/api/repos";
import { CloneRepos } from "@/features/repos/components/clone-repos";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/clone")({
  pendingMs: stepPendingMs,
  validateSearch: StepSearch,
  loaderDeps: ({ search }) => ({ machine: search.machine }),
  loader: async ({ context: { queries }, deps }) => {
    const machine = await stepMachine(queries, deps.machine);
    const accounts = await queries.query({
      ...accountsQuery(machine),
      staleTime: "static",
    });
    const hosts = accounts.map((account) => account.host);

    await Promise.allSettled(
      hosts.map((host) =>
        queries.query({
          ...remoteReposQuery(machine, host),
          staleTime: "static",
        }),
      ),
    );

    return { machine, hosts };
  },
  component: function CloneRoute() {
    const { machine, hosts } = Route.useLoaderData();
    const navigate = useNavigate();
    const { finish, finishing } = useFinishOnboarding();

    return (
      <CloneRepos
        machine={machine}
        hosts={hosts}
        onBack={() =>
          void navigate({
            to: "/onboarding/repos",
            search: { machine: machine.id },
          })
        }
        finishing={finishing}
        onContinue={finish}
      />
    );
  },
});
