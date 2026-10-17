import { createFileRoute } from "@tanstack/react-router";

import { Centered } from "@/components/centered";
import { Home } from "@/features/home/components/home";

export const Route = createFileRoute("/_onboarded/")({
  component: function HomeRoute() {
    return (
      <Centered>
        <Home />
      </Centered>
    );
  },
});
