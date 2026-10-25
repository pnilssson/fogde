import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import type { Machine } from "@/lib/machines";

export function useNotesChanges(machines: Machine[]) {
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.listen("notes.changed", () => {
        void queries.invalidateQueries({
          queryKey: [machine.id, "notes.list"],
        });
        void queries.invalidateQueries({
          queryKey: [machine.id, "notes.read"],
        });
      }),
    );

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, queries]);
}
