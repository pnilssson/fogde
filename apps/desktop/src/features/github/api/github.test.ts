import { QueryClient } from "@tanstack/react-query";
import { expect, test } from "vitest";

import {
  checkoutsQuery,
  diffQuery,
  follow,
  listQuery,
} from "@/features/github/api/github";
import type { Events, TerminalState } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

function setup() {
  const listeners = new Map<string, (params: unknown) => void>();
  const listen = (event: string, listener: (params: unknown) => void) => {
    listeners.set(event, listener);

    return () => {
      listeners.delete(event);
    };
  };
  const machine = { id: "local", client: { listen } } as unknown as Machine;
  const queries = new QueryClient();
  const repo = { host: "github.com", owner: "acme", name: "app" };
  const keys = {
    status: checkoutsQuery(machine, ["/repos/app"]).queryKey,
    uncommitted: diffQuery(machine, "/repos/app", "uncommitted").queryKey,
    turn: diffQuery(machine, "/repos/app", "turn", "t1").queryKey,
    other: diffQuery(machine, "/repos/app", "turn", "t2").queryKey,
    pulls: listQuery(machine, "pull", repo, "open").queryKey,
  };
  for (const key of Object.values(keys)) {
    queries.setQueryData(key, []);
  }

  const stop = follow(queries, machine);
  const emit = <E extends keyof Events>(event: E, params: Events[E]) => {
    listeners.get(event)?.(params);
  };
  const stale = () =>
    Object.fromEntries(
      Object.entries(keys).map(([name, key]) => [
        name,
        queries.getQueryState(key)?.isInvalidated,
      ]),
    );

  return { emit, stale, stop, listeners };
}

const state = (status: TerminalState["status"]): TerminalState => ({
  id: "t1",
  name: "Claude",
  agent: "claude",
  model: null,
  status,
  active: null,
  folder: "/repos/app",
  branch: "feature/login",
  changes: null,
});

test("refreshes the checkouts and their diffs when files change", () => {
  const { emit, stale } = setup();

  emit("workspaces.changed", null);

  expect(stale()).toEqual({
    status: true,
    uncommitted: true,
    turn: true,
    other: true,
    pulls: false,
  });
});

test("refreshes an agent's last turn once it stops working", () => {
  const { emit, stale } = setup();

  emit("terminals.state", state("working"));
  expect(stale().turn).toBe(false);

  emit("terminals.state", state("done"));
  expect(stale()).toEqual({
    status: false,
    uncommitted: false,
    turn: true,
    other: false,
    pulls: false,
  });
});

test("stops listening when stopped", () => {
  const { stop, listeners } = setup();

  stop();

  expect(listeners.size).toBe(0);
});
