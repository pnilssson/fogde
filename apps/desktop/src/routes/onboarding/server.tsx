import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { serverQuery, sshHostsQuery } from "@/features/server/api/server";
import { ConnectServer } from "@/features/server/components/connect-server";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/server")({
  pendingMs: stepPendingMs,
  loader: ({ context: { queries } }) =>
    Promise.all([
      queries.query({ ...serverQuery, staleTime: "static" }),
      queries.query({ ...sshHostsQuery, staleTime: "static" }),
    ]),
  component: function ServerRoute() {
    const navigate = useNavigate();

    return (
      <ConnectServer
        onBack={() => void navigate({ to: "/onboarding/where" })}
        onContinue={() => navigate({ to: "/onboarding/github" })}
      />
    );
  },
});
