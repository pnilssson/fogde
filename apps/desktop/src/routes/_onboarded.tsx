import { createFileRoute, redirect } from "@tanstack/react-router";

import { accountsQuery } from "@/features/github/api/accounts";
import { onboardedQuery } from "@/features/onboarding/api/onboarding";

export const Route = createFileRoute("/_onboarded")({
  beforeLoad: async ({ context }) => {
    if (await context.queries.query(onboardedQuery)) {
      return;
    }

    const accounts = await context.queries.query(accountsQuery);
    throw redirect({
      to: accounts.length === 0 ? "/onboarding/github" : "/onboarding/agents",
    });
  },
});
