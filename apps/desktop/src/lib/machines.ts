import { queryOptions } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";

import type { Machine as Entry } from "@/generated/protocol";
import type { Client } from "@/lib/client";
import { callOwn, own, serverClient } from "@/lib/host";

export interface Machine {
  id: string;
  name: string;
  local: boolean;
  client: Client;
}

export const computer: Machine = {
  id: "local",
  name: "This computer",
  local: isTauri(),
  client: own,
};

export const machinesQuery = queryOptions({
  queryKey: ["machines.list"],
  queryFn: () => callOwn("machines.list"),
});

const servers = new Map<string, Machine>();

export function entryId(entry: Entry) {
  return entry.kind === "computer" ? computer.id : entry.id;
}

export function machineOf(entry: Entry): Machine {
  if (entry.kind === "computer") {
    return computer;
  }

  const known = servers.get(entry.id);
  if (known?.name === entry.target) {
    return known;
  }

  const machine = {
    id: entry.id,
    name: entry.target,
    local: false,
    client: known?.client ?? serverClient(entry.id),
  };
  servers.set(entry.id, machine);

  return machine;
}

export function forget(id: string) {
  servers.get(id)?.client.reset();
  servers.delete(id);
}
