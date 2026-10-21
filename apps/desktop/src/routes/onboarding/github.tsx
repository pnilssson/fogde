import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { ConnectGitHub } from "@/features/github/components/connect-github";
import {
  StepSearch,
  stepMachine,
} from "@/features/onboarding/lib/step-machine";
import { isComputer } from "@/lib/machines";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/github")({
  pendingMs: stepPendingMs,
  validateSearch: StepSearch,
  loaderDeps: ({ search }) => ({ machine: search.machine }),
  loader: async ({ context: { queries }, deps }) => {
    const machine = await stepMachine(queries, deps.machine);
    await queries.query({ ...accountsQuery(machine), staleTime: "static" });

    return machine;
  },
  component: function GitHubRoute() {
    const machine = Route.useLoaderData();
    const navigate = useNavigate();

    return (
      <ConnectGitHub
        machine={machine}
        onBack={() =>
          void navigate({
            to: isComputer(machine)
              ? "/onboarding/where"
              : "/onboarding/server",
          })
        }
        onContinue={() =>
          navigate({
            to: "/onboarding/agents",
            search: { machine: machine.id },
          })
        }
      />
    );
  },
});
