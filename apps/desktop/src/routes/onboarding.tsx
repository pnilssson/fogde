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
import { stepIndex } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding")({
  loader: ({ context: { queries } }) => {
    void Promise.allSettled([
      queries.query({ ...accountsQuery, staleTime: "static" }),
      queries.query({ ...agentsQuery, staleTime: "static" }),
      queries.query({ ...reposFolderQuery, staleTime: "static" }),
    ]);
  },
  component: function OnboardingRoute() {
    const pathname = useRouterState({
      select: (state) => state.location.pathname,
    });

    // A layout effect, so a step can raise a toast as it opens.
    useLayoutEffect(() => {
      toast.close();
    }, [pathname]);

    return (
      <Centered>
        <div className="flex flex-col items-start gap-4">
          <div className="[view-transition-name:progress]">
            <OnboardingProgress position={stepIndex(pathname)} />
          </div>
          <div className="[view-transition-name:step]">
            <Outlet />
          </div>
        </div>
      </Centered>
    );
  },
});
