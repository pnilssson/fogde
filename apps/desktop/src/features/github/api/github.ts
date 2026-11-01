import {
  keepPreviousData,
  type QueryClient,
  queryOptions,
} from "@tanstack/react-query";

import type { DiffScope, Filter, Repo, Status } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const refresh = 60_000;

export type Kind = "pull" | "issue";

export const checkoutsQuery = (machine: Machine, folders: string[]) =>
  queryOptions({
    queryKey: [machine.id, "checkouts.status", folders],
    queryFn: () => machine.client.call("checkouts.status", folders),
    refetchInterval: refresh,
    placeholderData: keepPreviousData,
  });

export const listQuery = (
  machine: Machine,
  kind: Kind,
  repo: Repo,
  filter: Filter,
) =>
  queryOptions({
    queryKey: [
      machine.id,
      kind === "pull" ? "github.pulls" : "github.issues",
      repo,
      filter,
    ],
    queryFn: () =>
      machine.client.call(kind === "pull" ? "github.pulls" : "github.issues", {
        repo,
        filter,
      }),
    refetchInterval: refresh,
  });

export const detailQuery = (
  machine: Machine,
  kind: Kind,
  repo: Repo,
  number: number,
) =>
  queryOptions({
    queryKey: [
      machine.id,
      kind === "pull" ? "github.pull" : "github.issue",
      repo,
      number,
    ],
    queryFn: () =>
      machine.client.call(kind === "pull" ? "github.pull" : "github.issue", {
        repo,
        number,
      }),
    refetchInterval: refresh,
  });

export const optionsQuery = (machine: Machine, repo: Repo) =>
  queryOptions({
    queryKey: [machine.id, "github.options", repo],
    queryFn: () => machine.client.call("github.options", repo),
    staleTime: Infinity,
  });

export const branchesQuery = (machine: Machine, path: string) =>
  queryOptions({
    queryKey: [machine.id, "checkouts.branches", path],
    queryFn: () => machine.client.call("checkouts.branches", path),
  });

export const diffQuery = (
  machine: Machine,
  path: string,
  scope: DiffScope,
  terminal: string | null = null,
) =>
  queryOptions({
    queryKey: [machine.id, "checkouts.diff", path, scope, terminal],
    queryFn: () =>
      machine.client.call("checkouts.diff", { path, scope, terminal }),
    refetchInterval: refresh,
  });

export const logQuery = (machine: Machine, repo: Repo, job: number) =>
  queryOptions({
    queryKey: [machine.id, "github.log", repo, job],
    queryFn: () => machine.client.call("github.log", { repo, job }),
    staleTime: Infinity,
  });

export function follow(queries: QueryClient, machine: Machine) {
  const statuses = new Map<string, Status>();
  const stops = [
    machine.client.listen("workspaces.changed", () => {
      void queries.invalidateQueries({
        predicate: ({ queryKey: [id, method] }) =>
          id === machine.id &&
          (method === "checkouts.status" || method === "checkouts.diff"),
      });
    }),
    machine.client.listen("terminals.state", ({ id, status }) => {
      const before = statuses.get(id);
      statuses.set(id, status);
      if (status === "working" || before === status) {
        return;
      }

      void queries.invalidateQueries({
        queryKey: [machine.id, "checkouts.diff"],
        predicate: ({ queryKey }) =>
          queryKey[3] === "turn" && queryKey[4] === id,
      });
    }),
    machine.client.listen("terminals.ended", ({ id }) => {
      statuses.delete(id);
    }),
  ];

  return () => {
    for (const stop of stops) {
      stop();
    }
  };
}
