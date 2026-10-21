import { useMutation, useQueryClient } from "@tanstack/react-query";

import { callOwn } from "@/lib/host";
import { idOf, machinesQuery, type Machine } from "@/lib/machines";

export function useRenameMachine(machine: Machine, onRenamed: () => void) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: (name: string) =>
      callOwn("machines.rename", { machine: idOf(machine), name }),
    onSuccess: async () => {
      await queries.invalidateQueries(machinesQuery);
      onRenamed();
    },
  });
}
