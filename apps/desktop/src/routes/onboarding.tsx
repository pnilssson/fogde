import {
  Outlet,
  createFileRoute,
  useRouterState,
  useSearch,
} from "@tanstack/react-router";
import { useLayoutEffect } from "react";

import { Centered } from "@/components/centered";
import { OnboardingProgress } from "@/components/onboarding-progress";
import { ThemeSwitcher } from "@/components/theme-switcher";
import { toast } from "@/components/ui/toast";
import { computer, machinesQuery } from "@/lib/machines";
import { stepOf, steps } from "@/lib/onboarding";

export const Route = createFileRoute("/onboarding")({
  loader: ({ context: { queries } }) => {
    void queries.query(machinesQuery).catch(() => undefined);
  },
  component: function OnboardingRoute() {
    const pathname = useRouterState({
      select: (state) => (state.resolvedLocation ?? state.location).pathname,
    });
    const { machine } = useSearch({ strict: false });
    const step = stepOf(pathname);
    const shown = steps(
      step === "server" || (machine !== undefined && machine !== computer.id),
    );

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
        {import.meta.env.DEV && (
          <div className="fixed bottom-4 left-4">
            <ThemeSwitcher />
          </div>
        )}
      </Centered>
    );
  },
});
