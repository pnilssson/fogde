import { useQuery } from "@tanstack/react-query";

import { optionsQuery } from "@/features/github/api/github";
import { nextStep, syncOf } from "@/features/github/lib/sync";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export function usePlan(
  machine: Machine,
  checkout: CheckoutStatus,
  hasPull: boolean,
  online: boolean,
) {
  const { repo, branch } = checkout;
  const options = useQuery({
    ...optionsQuery(machine, repo ?? { host: "", owner: "", name: "" }),
    enabled: online && repo !== null,
  });
  const base = options.data?.branch ?? null;
  const canOpen = online && branch !== null && base !== null && branch !== base;

  return {
    canOpen,
    sync: syncOf(checkout),
    next: nextStep(checkout, hasPull, canOpen),
  };
}
