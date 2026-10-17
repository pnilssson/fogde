import { createFileRoute, redirect } from "@tanstack/react-router";

import { Centered } from "@/components/centered";
import { userQuery } from "@/features/github/api/user";
import { SignIn } from "@/features/github/components/sign-in";

export const Route = createFileRoute("/sign-in")({
  beforeLoad: async ({ context }) => {
    const user = await context.queries.query({
      ...userQuery,
      staleTime: "static",
    });
    if (user) {
      throw redirect({ to: "/" });
    }
  },
  component: function SignInRoute() {
    return (
      <Centered>
        <SignIn />
      </Centered>
    );
  },
});
