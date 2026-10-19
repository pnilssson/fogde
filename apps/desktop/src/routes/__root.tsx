import type { QueryClient } from "@tanstack/react-query";
import {
  Outlet,
  createRootRouteWithContext,
  useRouterState,
} from "@tanstack/react-router";

import { SidebarToggle } from "@/components/sidebar-toggle";
import { TitleBar } from "@/components/title-bar";
import { Separator } from "@/components/ui/separator";
import { SidebarProvider } from "@/components/ui/sidebar";

export const Route = createRootRouteWithContext<{ queries: QueryClient }>()({
  component: function RootRoute() {
    const onboarded = useRouterState({
      select: (state) =>
        state.matches.some((match) => match.routeId === "/_onboarded"),
    });

    return (
      <SidebarProvider className="h-screen min-h-0 flex-col">
        <TitleBar>{onboarded && <SidebarToggle />}</TitleBar>
        {onboarded && <Separator />}
        <div className="min-h-0 flex-1">
          <Outlet />
        </div>
      </SidebarProvider>
    );
  },
});
