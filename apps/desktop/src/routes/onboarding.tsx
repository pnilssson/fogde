import { useQuery } from "@tanstack/react-query";
import {
  Outlet,
  createFileRoute,
  useRouterState,
} from "@tanstack/react-router";
import { useLayoutEffect } from "react";

import { Centered } from "@/components/centered";
import { OnboardingProgress } from "@/components/onboarding-progress";
import { toast } from "@/components/ui/toast";
import { agentsQuery } from "@/features/agents/api/agents";
import { accountsQuery } from "@/features/github/api/accounts";
import { reposFolderQuery } from "@/features/repos/api/repos";
import { serverQuery } from "@/features/server/api/server";
import { stepOf, steps } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding")({
  loader: ({ context: { queries }, location }) => {
    void queries.query({ ...serverQuery, staleTime: "static" });
    const step = stepOf(location.pathname);
    if (step === "where" || step === "server") {
      return;
    }
    void Promise.allSettled([
      queries.query({ ...accountsQuery, staleTime: "static" }),
      queries.query({ ...agentsQuery, staleTime: "static" }),
      queries.query({ ...reposFolderQuery, staleTime: "static" }),
    ]);
  },
  component: function OnboardingRoute() {
    const pathname = useRouterState({
      select: (state) => (state.resolvedLocation ?? state.location).pathname,
    });
    const { data: server } = useQuery(serverQuery);
    const step = stepOf(pathname);
    const shown = steps(step === "server" || (server ?? null) !== null);

    useLayoutEffect(() => {
      toast.close();
    }, [pathname]);

    return (
      <Centered>
        <div className="flex flex-col items-start gap-4">
          <div className="[view-transition-name:progress]">
            <OnboardingProgress
              position={step ? shown.indexOf(step) : 0}
              total={shown.length}
            />
          </div>
          <div className="[view-transition-name:step]">
            <Outlet />
          </div>
        </div>
      </Centered>
    );
  },
});
