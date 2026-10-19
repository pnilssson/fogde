import { ConnectAgents } from "@/features/agents/components/connect-agents";
import {
  StepSearch,
  stepMachine,
} from "@/features/onboarding/lib/step-machine";
import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { agentsQuery } from "@/lib/agents";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/agents")({
  pendingMs: stepPendingMs,
  validateSearch: StepSearch,
  loaderDeps: ({ search }) => ({ machine: search.machine }),
  loader: async ({ context: { queries }, deps }) => {
    const machine = await stepMachine(queries, deps.machine);
    await queries.query({ ...agentsQuery(machine), staleTime: "static" });

    return machine;
  },
  component: function AgentsRoute() {
    const machine = Route.useLoaderData();
    const navigate = useNavigate();
    const search = { machine: machine.id };

    return (
      <ConnectAgents
        machine={machine}
        onBack={() => void navigate({ to: "/onboarding/github", search })}
        onContinue={() => navigate({ to: "/onboarding/repos", search })}
      />
    );
  },
});
