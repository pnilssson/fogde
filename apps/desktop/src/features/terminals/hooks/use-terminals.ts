import {
  type UseQueryResult,
  useMutation,
  useQueries,
  useQueryClient,
} from "@tanstack/react-query";
import { useEffect, useSyncExternalStore } from "react";

import { toast } from "@/components/ui/toast";
import {
  addTerminal,
  removeTerminal,
  workspacesQuery,
} from "@/features/terminals/api/workspaces";
import type {
  CreateTerminalParams,
  Terminal,
  Workspace,
} from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export function useGroups(machines: Machine[]) {
  const results = useQueries({ queries: machines.map(workspacesQuery) });
  const queries = useQueryClient();

  useEffect(() => {
    const stops = machines.map((machine) =>
      machine.client.subscribe(() => {
        if (machine.client.getSnapshot().state === "connected") {
          void queries.invalidateQueries(workspacesQuery(machine), {
            cancelRefetch: false,
          });
        }
      }),
    );

    return () => {
      for (const stop of stops) {
        stop();
      }
    };
  }, [machines, queries]);

  return results.flatMap((workspaces, index) => {
    const machine = machines[index];

    return machine ? [{ machine, workspaces }] : [];
  });
}

export function useConnection(machine: Machine) {
  return useSyncExternalStore(
    machine.client.subscribe,
    machine.client.getSnapshot,
  );
}

export function useStatus(
  machine: Machine,
  workspaces: UseQueryResult<Workspace[]>,
) {
  const connection = useConnection(machine);
  const queries = useQueryClient();
  const disconnected = connection.state === "disconnected";
  const failed = connection.state === "connected" && workspaces.isError;

  return {
    connected: connection.state === "connected",
    disconnected,
    failed,
    retrying: failed && workspaces.isFetching,
    error: disconnected
      ? (connection.error ?? undefined)
      : failed
        ? workspaces.error.message
        : undefined,
    recover: () => {
      if (machine.client.getSnapshot().state === "connected") {
        void queries.invalidateQueries(workspacesQuery(machine));
      } else {
        void machine.client.reconnect().catch(() => undefined);
      }
    },
  };
}

export function useCreateTerminal(
  machine: Machine,
  onCreated: (terminal: Terminal) => void,
) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: (params: CreateTerminalParams) =>
      machine.client.call("terminals.create", params),
    onSuccess: (terminal, { workspace }) => {
      addTerminal(queries, machine, workspace, terminal);
      onCreated(terminal);
    },
    onError: (error) => {
      toast.add({
        type: "error",
        title: "Could not open a terminal",
        description: error.message,
      });
    },
  });
}

export function useCloseTerminal(machine: Machine) {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: (terminal: Terminal) =>
      machine.client.call("terminals.close", terminal.id),
    onSuccess: (_, closed) => {
      removeTerminal(queries, machine, closed);
    },
    onError: (error, terminal) => {
      toast.add({
        type: "error",
        title: `Could not close ${terminal.name}`,
        description: error.message,
      });
    },
  });
}
