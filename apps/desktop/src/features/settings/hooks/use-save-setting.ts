import {
  type DataTag,
  type QueryKey,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";

import { toast } from "@/components/ui/toast";

interface Setting<T> {
  queryKey: DataTag<QueryKey, T, Error>;
}

export function useSaveSetting<T>(
  setting: Setting<T>,
  save: (next: T) => Promise<unknown>,
  failure: string,
) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: save,
    onMutate: (next: T) => {
      const before = queries.getQueryData(setting.queryKey);
      queries.setQueryData(setting.queryKey, next);

      return before;
    },
    onError: (error, _, before) => {
      queries.setQueryData(setting.queryKey, before);
      toast.add({ type: "error", title: failure, description: error.message });
    },
  });
}
