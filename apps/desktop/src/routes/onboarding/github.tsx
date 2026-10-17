import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { Centered } from "@/components/centered";
import { ConnectGitHub } from "@/features/github/components/connect-github";

export const Route = createFileRoute("/onboarding/github")({
  component: function GitHubRoute() {
    const navigate = useNavigate();

    return (
      <Centered>
        <ConnectGitHub
          onContinue={() => void navigate({ to: "/onboarding/agents" })}
        />
      </Centered>
    );
  },
});
