import {
  type UseQueryResult,
  useIsMutating,
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useCallback } from "react";

import {
  addTerminal,
  removeTerminal,
  workspacesQuery,
} from "@/features/workspaces/api/workspaces";
import { opened } from "@/features/workspaces/lib/opening";
import { useConnected } from "@/hooks/use-connected";
import { useConnection } from "@/hooks/use-connection";
import type {
  Agent,
  CreateTerminalParams,
  Terminal,
  Workspace,
} from "@/generated/protocol";
import { agentsQuery } from "@/lib/agents";
import { report } from "@/lib/report";
import type { Machine } from "@/lib/machines";

export function useGroups(machines: Machine[]) {
  const queries = useQueryClient();
  const combine = useCallback(
    (results: UseQueryResult<Workspace[]>[]) =>
      results.flatMap((workspaces, index) => {
        const machine = machines[index];

        return machine ? [{ machine, workspaces }] : [];
      }),
    [machines],
  );

  useConnected(machines, (machine) => {
    void queries.invalidateQueries(workspacesQuery(machine), {
      cancelRefetch: false,
    });
  });

  return useQueries({ queries: machines.map(workspacesQuery), combine });
}

export function useMachineStatus(
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

export function useChoices(machine: Machine): (Agent | null)[] | undefined {
  const { data: agents = [], isPending } = useQuery({
    ...agentsQuery(machine),
    staleTime: Infinity,
  });
  if (isPending) {
    return undefined;
  }

  return [
    null,
    ...agents
      .filter(({ state }) => state === "signedIn")
      .map(({ agent }) => agent),
  ];
}

interface Create extends CreateTerminalParams {
  machine: Machine;
}

export function useCreateTerminal() {
  const queries = useQueryClient();

  return useMutation({
    mutationFn: ({ machine, ...params }: Create) =>
      machine.client.call("terminals.create", params),
    onSuccess: (terminal, { machine, workspace }) => {
      addTerminal(queries, machine, workspace, terminal);
      opened({ machine: machine.id, id: terminal.id });
    },
    onError: (error) => {
      report("Could not open a terminal", error);
    },
  });
}

function closeKey(machine: Machine) {
  return [machine.id, "terminals.close"];
}

export function useClosing(machine: Machine, terminal: Terminal) {
  return (
    useIsMutating({
      mutationKey: closeKey(machine),
      predicate: ({ state: { variables } }) =>
        typeof variables === "object" &&
        variables !== null &&
        "id" in variables &&
        variables.id === terminal.id,
    }) > 0
  );
}

export function useCloseTerminal(machine: Machine) {
  const queries = useQueryClient();

  return useMutation({
    mutationKey: closeKey(machine),
    mutationFn: (terminal: Terminal) =>
      machine.client.call("terminals.close", terminal.id),
    onSuccess: (_, closed) => {
      removeTerminal(queries, machine, closed);
    },
    onError: (error, terminal) => {
      report(`Could not close ${terminal.name}`, error);
    },
  });
}
