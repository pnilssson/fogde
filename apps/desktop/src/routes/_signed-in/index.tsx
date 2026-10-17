import { createFileRoute } from "@tanstack/react-router";

import { Centered } from "@/components/centered";
import { Home } from "@/features/home/components/home";

export const Route = createFileRoute("/_signed-in/")({
  component: function HomeRoute() {
    const { user } = Route.useRouteContext();

    return (
      <Centered>
        <Home user={user} />
      </Centered>
    );
  },
});
