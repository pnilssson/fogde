import { useMutation, useQueryClient } from "@tanstack/react-query";

import { toast } from "@/components/ui/toast";
import { callOwn } from "@/lib/host";
import { forget, idOf, machinesQuery, type Machine } from "@/lib/machines";

type Change = "connect" | "disconnect" | "remove";

const failures: Record<Change, string> = {
  connect: "Could not connect",
  disconnect: "Could not disconnect",
  remove: "Could not remove",
};

export function useChangeMachine(change: Change, onChanged?: () => void) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: (machine: Machine) =>
      callOwn(`machines.${change}`, idOf(machine)),
    onSuccess: async (_, machine) => {
      if (change !== "connect") {
        forget(machine.id);
        queries.removeQueries({ queryKey: [machine.id] });
      }

      await queries.invalidateQueries(machinesQuery);
      onChanged?.();
    },
    onError: (error, machine) => {
      toast.add({
        type: "error",
        title: `${failures[change]} ${machine.name}`,
        description: error.message,
      });
    },
  });
}
