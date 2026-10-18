import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { ConnectGitHub } from "@/features/github/components/connect-github";

export const Route = createFileRoute("/onboarding/github")({
  loader: ({ context: { queries } }) =>
    queries.query({ ...accountsQuery, staleTime: "static" }),
  component: function GitHubRoute() {
    const navigate = useNavigate();

    return (
      <ConnectGitHub
        onContinue={() => void navigate({ to: "/onboarding/agents" })}
      />
    );
  },
});
