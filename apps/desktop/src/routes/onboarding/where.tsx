import { onboardedQuery } from "@/features/onboarding/api/onboarding";
import { ChooseWhere } from "@/features/server/components/choose-where";
import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { computer } from "@/lib/machines";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/where")({
  pendingMs: stepPendingMs,
  loader: ({ context: { queries } }) => queries.query(onboardedQuery),
  component: function WhereRoute() {
    const onboarded = Route.useLoaderData();
    const navigate = useNavigate();

    return (
      <ChooseWhere
        onboarded={onboarded}
        onCancel={() => void navigate({ to: "/" })}
        onComputer={() =>
          navigate({
            to: "/onboarding/github",
            search: { machine: computer.id },
          })
        }
        onServer={() => navigate({ to: "/onboarding/server" })}
      />
    );
  },
});
