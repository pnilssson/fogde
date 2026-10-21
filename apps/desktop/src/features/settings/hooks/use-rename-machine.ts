import { useMutation, useQueryClient } from "@tanstack/react-query";

import { toast } from "@/components/ui/toast";
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
    onError: (error) => {
      toast.add({
        type: "error",
        title: `Could not rename ${machine.name}`,
        description: error.message,
      });
    },
  });
}
