import { queryOptions } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";

export const accessibilityQuery = queryOptions({
  queryKey: ["accessibility.get"],
  queryFn: () => callOwn("accessibility.get"),
});
