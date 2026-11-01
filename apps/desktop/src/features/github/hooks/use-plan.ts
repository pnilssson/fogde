import { useQuery } from "@tanstack/react-query";

import { optionsQuery } from "@/features/github/api/github";
import { nowhere } from "@/features/github/lib/scope";
import { nextStep, openable, syncOf } from "@/features/github/lib/sync";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export function usePlan(
  machine: Machine,
  checkout: CheckoutStatus,
  hasPull: boolean,
  online: boolean,
) {
  const { repo } = checkout;
  const options = useQuery({
    ...optionsQuery(machine, repo ?? nowhere),
    enabled: online && repo !== null,
  });
  const base = options.data?.branch ?? null;
  const canOpen = online && openable(checkout, base);

  return {
    canOpen,
    base,
    sync: syncOf(checkout),
    next: nextStep(checkout, hasPull, canOpen),
  };
}
