import { createFileRoute, redirect } from "@tanstack/react-router";

import { agentsQuery } from "@/features/agents/api/agents";
import { accountsQuery } from "@/features/github/api/accounts";
import { onboardedQuery } from "@/features/onboarding/api/onboarding";

export const Route = createFileRoute("/_onboarded")({
  beforeLoad: async ({ context }) => {
    if (await context.queries.query(onboardedQuery)) {
      return;
    }

    const accounts = await context.queries.query(accountsQuery);
    if (accounts.length === 0) {
      throw redirect({ to: "/onboarding/github" });
    }

    const agents = await context.queries.query(agentsQuery);
    throw redirect({
      to: agents.some((status) => status.state === "signedIn")
        ? "/onboarding/repos"
        : "/onboarding/agents",
    });
  },
});
