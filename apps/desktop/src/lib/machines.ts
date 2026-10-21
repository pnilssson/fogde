import { queryOptions } from "@tanstack/react-query";
import { isTauri } from "@tauri-apps/api/core";

import type { Machine as Entry, MachineId } from "@/generated/protocol";
import type { Client } from "@/lib/client";
import { callOwn, own, serverClient } from "@/lib/host";

export interface Machine {
  id: string;
  name: string;
  target?: string;
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
let named = computer;

export function entryId(entry: Entry) {
  return entry.kind === "computer" ? computer.id : entry.id;
}

export function isComputer(machine: Machine) {
  return machine.id === computer.id;
}

export function idOf(machine: Machine): MachineId {
  return isComputer(machine)
    ? { kind: "computer" }
    : { kind: "server", id: machine.id };
}

export function machineOf(entry: Entry): Machine {
  if (entry.kind === "computer") {
    if (named.name !== entry.name) {
      named = { ...computer, name: entry.name };
    }

    return named;
  }

  const known = servers.get(entry.id);
  if (known?.name === entry.name && known.target === entry.target) {
    return known;
  }

  const machine = {
    id: entry.id,
    name: entry.name,
    target: entry.target,
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
