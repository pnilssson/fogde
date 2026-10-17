import { createFileRoute, redirect } from "@tanstack/react-router";

import { userQuery } from "@/features/github/api/user";

export const Route = createFileRoute("/_signed-in")({
  beforeLoad: async ({ context }) => {
    const user = await context.queries.query({
      ...userQuery,
      staleTime: "static",
    });
    if (!user) {
      throw redirect({ to: "/sign-in" });
    }

    return { user };
  },
});
