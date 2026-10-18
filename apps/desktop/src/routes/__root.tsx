import type { QueryClient } from "@tanstack/react-query";
import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";

import { ThemeSwitcher } from "@/components/theme-switcher";
import { TitleBar } from "@/components/title-bar";

export const Route = createRootRouteWithContext<{ queries: QueryClient }>()({
  component: function RootRoute() {
    return (
      <div className="flex h-screen flex-col">
        <TitleBar />
        <div className="min-h-0 flex-1">
          <Outlet />
        </div>
        {import.meta.env.DEV && <ThemeSwitcher />}
      </div>
    );
  },
});
