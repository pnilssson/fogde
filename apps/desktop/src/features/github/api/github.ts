import { keepPreviousData, queryOptions } from "@tanstack/react-query";

import type { DiffScope, Filter, Repo } from "@/generated/protocol";
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

export const diffQuery = (machine: Machine, path: string, scope: DiffScope) =>
  queryOptions({
    queryKey: [machine.id, "checkouts.diff", path, scope],
    queryFn: () => machine.client.call("checkouts.diff", { path, scope }),
    refetchInterval: refresh,
  });

export const logQuery = (machine: Machine, repo: Repo, job: number) =>
  queryOptions({
    queryKey: [machine.id, "github.log", repo, job],
    queryFn: () => machine.client.call("github.log", { repo, job }),
    staleTime: Infinity,
  });
