import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { Centered } from "@/components/centered";
import { ConnectAgents } from "@/features/agents/components/connect-agents";
import { useFinishOnboarding } from "@/features/onboarding/hooks/use-finish-onboarding";

export const Route = createFileRoute("/onboarding/agents")({
  component: function AgentsRoute() {
    const navigate = useNavigate();
    const finish = useFinishOnboarding();

    return (
      <Centered>
        <ConnectAgents
          onBack={() => void navigate({ to: "/onboarding/github" })}
          onContinue={finish}
        />
      </Centered>
    );
  },
});
