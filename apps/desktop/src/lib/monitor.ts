import { queryOptions } from "@tanstack/react-query";

import type { Bar, Metric } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { isComputer, type Machine } from "@/lib/machines";

export const barQuery = queryOptions({
  queryKey: ["monitor.getBar"],
  queryFn: () => callOwn("monitor.getBar"),
  staleTime: Infinity,
});

export const metrics: { metric: Metric; label: string }[] = [
  { metric: "cpu", label: "Fogde CPU" },
  { metric: "memory", label: "Fogde memory" },
  { metric: "battery", label: "Battery drain" },
  { metric: "latency", label: "Connection latency" },
];

export function barMachine(bar: Bar, machines: Machine[]) {
  const { machine } = bar;
  const chosen =
    machine.kind === "server"
      ? machines.find(({ id }) => id === machine.id)
      : undefined;

  return chosen ?? machines.find(isComputer);
}
