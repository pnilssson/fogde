import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  RouterProvider,
  createHashHistory,
  createRouter,
} from "@tanstack/react-router";
import { LazyMotion, MotionConfig, domAnimation } from "motion/react";

import { Failure } from "@/components/failure";
import { Loading } from "@/components/loading";
import { Toaster } from "@/components/ui/toast";
import { TooltipProvider } from "@/components/ui/tooltip";
import { routeTree } from "@/generated/route-tree.gen";
import { stepTransition } from "@/lib/onboarding";

const queries = new QueryClient({
  defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false } },
});

const router = createRouter({
  routeTree,
  history: createHashHistory(),
  context: { queries },
  defaultPendingComponent: Loading,
  defaultErrorComponent: Failure,
  defaultViewTransition: {
    types: ({ fromLocation, toLocation }) =>
      stepTransition(fromLocation?.pathname, toLocation.pathname),
  },
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export function App() {
  return (
    <QueryClientProvider client={queries}>
      <LazyMotion features={domAnimation} strict>
        <MotionConfig reducedMotion="user">
          <Toaster>
            <TooltipProvider delay={600}>
              <RouterProvider router={router} />
            </TooltipProvider>
          </Toaster>
        </MotionConfig>
      </LazyMotion>
    </QueryClientProvider>
  );
}
