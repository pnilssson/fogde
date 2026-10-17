import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  RouterProvider,
  createHashHistory,
  createRouter,
} from "@tanstack/react-router";

import { Failure } from "@/components/failure";
import { Loading } from "@/components/loading";
import { routeTree } from "@/generated/route-tree.gen";

const queries = new QueryClient({
  defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false } },
});

const router = createRouter({
  routeTree,
  history: createHashHistory(),
  context: { queries },
  defaultPendingComponent: Loading,
  defaultErrorComponent: Failure,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export function App() {
  return (
    <QueryClientProvider client={queries}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
