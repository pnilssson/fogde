import { queryOptions } from "@tanstack/react-query";

import { call } from "@/lib/host";

export const onboardedQuery = queryOptions({
  queryKey: ["onboarding.done"],
  queryFn: () => call("onboarding.done"),
  staleTime: "static",
});
