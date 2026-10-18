import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { agentsQuery } from "@/features/agents/api/agents";
import { ConnectAgents } from "@/features/agents/components/connect-agents";

export const Route = createFileRoute("/onboarding/agents")({
  loader: ({ context: { queries } }) =>
    queries.query({ ...agentsQuery, staleTime: "static" }),
  component: function AgentsRoute() {
    const navigate = useNavigate();

    return (
      <ConnectAgents
        onBack={() => void navigate({ to: "/onboarding/github" })}
        onContinue={() => void navigate({ to: "/onboarding/repos" })}
      />
    );
  },
});
