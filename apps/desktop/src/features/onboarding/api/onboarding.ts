import { queryOptions } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";

export const onboardedQuery = queryOptions({
  queryKey: ["onboarding.done"],
  queryFn: () => callOwn("onboarding.done"),
  staleTime: "static",
  meta: { own: true },
});
