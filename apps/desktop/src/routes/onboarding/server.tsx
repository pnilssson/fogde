import { createFileRoute, useNavigate } from "@tanstack/react-router";

import { sshHostsQuery } from "@/features/server/api/server";
import { ConnectServer } from "@/features/server/components/connect-server";
import { stepPendingMs } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding/server")({
  pendingMs: stepPendingMs,
  loader: ({ context: { queries } }) =>
    queries.query({ ...sshHostsQuery, staleTime: "static" }),
  component: function ServerRoute() {
    const navigate = useNavigate();

    return (
      <ConnectServer
        onBack={() => void navigate({ to: "/onboarding/where" })}
        onContinue={(machine) =>
          navigate({ to: "/onboarding/github", search: { machine } })
        }
      />
    );
  },
});
