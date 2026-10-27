import { useSyncExternalStore } from "react";

import type { Machine } from "@/lib/machines";

export const statusColors = {
  connected: "bg-success",
  disconnected: "bg-destructive",
  connecting: "bg-muted-foreground",
  idle: "bg-muted-foreground",
  off: "bg-muted-foreground",
};

export const statusLabels = {
  connected: "Connected",
  disconnected: "Not connected",
  connecting: "Connecting",
  idle: "Not connected",
  off: "Disconnected",
};

export type Status = keyof typeof statusColors;

export function useStatus(machine: Machine, enabled: boolean): Status {
  const { state } = useSyncExternalStore(
    machine.client.subscribe,
    machine.client.getSnapshot,
  );

  return enabled ? state : "off";
}
