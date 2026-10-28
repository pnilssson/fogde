import type { QueryClient } from "@tanstack/react-query";
import {
  Outlet,
  createRootRouteWithContext,
  useRouterState,
} from "@tanstack/react-router";

import { TitleBar } from "@/components/title-bar";
import { SidebarProvider } from "@/components/ui/sidebar";

export const Route = createRootRouteWithContext<{ queries: QueryClient }>()({
  component: function RootRoute() {
    const onboarded = useRouterState({
      select: (state) =>
        state.matches.some((match) => match.routeId === "/_onboarded"),
    });

    return (
      <SidebarProvider className="h-screen min-h-0 flex-col">
        {!onboarded && <TitleBar />}
        <div className="min-h-0 flex-1">
          <Outlet />
        </div>
      </SidebarProvider>
    );
  },
});
