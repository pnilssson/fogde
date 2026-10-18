import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { agentsQuery } from "@/features/agents/api/agents";
import { ConnectAgents } from "@/features/agents/components/connect-agents";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/agents")({
  pendingMs: stepPendingMs,
  loader: ({ context: { queries } }) =>
    queries.query({ ...agentsQuery, staleTime: "static" }),
  component: function AgentsRoute() {
    const navigate = useNavigate();

    return (
      <ConnectAgents
        onBack={() => void navigate({ to: "/onboarding/github" })}
        onContinue={() => navigate({ to: "/onboarding/repos" })}
      />
    );
  },
});
