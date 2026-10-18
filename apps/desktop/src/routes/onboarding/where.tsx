import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { serverQuery } from "@/features/server/api/server";
import { ChooseWhere } from "@/features/server/components/choose-where";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/where")({
  pendingMs: stepPendingMs,
  loader: ({ context: { queries } }) =>
    queries.query({ ...serverQuery, staleTime: "static" }),
  component: function WhereRoute() {
    const navigate = useNavigate();

    return (
      <ChooseWhere
        onComputer={() => navigate({ to: "/onboarding/github" })}
        onServer={() => navigate({ to: "/onboarding/server" })}
      />
    );
  },
});
