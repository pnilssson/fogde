import type { QueryClient } from "@tanstack/react-query";
import { redirect } from "@tanstack/react-router";
import { z } from "zod";

import { entryId, machineOf, machinesQuery } from "@/lib/machines";

export const StepSearch = z.object({ machine: z.string() });

export async function stepMachine(queries: QueryClient, id: string) {
  const entries = await queries.query(machinesQuery);
  const entry = entries.find((known) => entryId(known) === id);
  if (!entry) {
    throw redirect({ to: "/onboarding/where" });
  }

  return machineOf(entry);
}
