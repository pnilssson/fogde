import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { follow } from "@/features/github/api/github";
import type { Machine } from "@/lib/machines";

export function useFollow(machines: Machine[]) {
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) => follow(queries, machine));

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [queries, machines]);
}
