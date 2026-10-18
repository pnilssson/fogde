import { createFileRoute, redirect } from "@tanstack/react-router";

import { agentsQuery } from "@/features/agents/api/agents";
import { accountsQuery } from "@/features/github/api/accounts";
import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { serverQuery } from "@/features/server/api/server";

export const Route = createFileRoute("/_onboarded")({
  beforeLoad: async ({ context }) => {
    if (await context.queries.query(onboardedQuery)) {
      return;
    }

    if ((await context.queries.query(serverQuery)) !== null) {
      throw redirect({ to: "/onboarding/where" });
    }

    const accounts = await context.queries.query(accountsQuery);
    if (accounts.length === 0) {
      throw redirect({ to: "/onboarding/where" });
    }

    const agents = await context.queries.query(agentsQuery);
    throw redirect({
      to: agents.some((status) => status.state === "signedIn")
        ? "/onboarding/repos"
        : "/onboarding/agents",
    });
  },
});
