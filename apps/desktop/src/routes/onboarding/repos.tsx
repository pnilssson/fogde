import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import {
  StepSearch,
  stepMachine,
} from "@/features/onboarding/lib/step-machine";
import { remoteReposQuery } from "@/features/repos/api/repos";
import { ChooseFolder } from "@/features/repos/components/choose-folder";
import { reposFolderQuery } from "@/lib/folders";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/repos")({
  pendingMs: stepPendingMs,
  validateSearch: StepSearch,
  loaderDeps: ({ search }) => ({ machine: search.machine }),
  loader: async ({ context: { queries }, deps }) => {
    const machine = await stepMachine(queries, deps.machine);
    void queries
      .query({ ...accountsQuery(machine), staleTime: "static" })
      .then((accounts) =>
        Promise.allSettled(
          accounts.map((account) =>
            queries.query({
              ...remoteReposQuery(machine, account.host),
              staleTime: "static",
            }),
          ),
        ),
      )
      .catch(() => undefined);
    await queries.query({ ...reposFolderQuery(machine), staleTime: "static" });

    return machine;
  },
  component: function ReposRoute() {
    const machine = Route.useLoaderData();
    const navigate = useNavigate();
    const search = { machine: machine.id };

    return (
      <ChooseFolder
        machine={machine}
        onBack={() => void navigate({ to: "/onboarding/agents", search })}
        onContinue={() => navigate({ to: "/onboarding/clone", search })}
      />
    );
  },
});
