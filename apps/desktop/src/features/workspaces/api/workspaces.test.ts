import { QueryClient, QueryObserver } from "@tanstack/react-query";
import { expect, test, vi } from "vitest";

import {
  addTerminal,
  applyState,
  removeTerminal,
  workspacesQuery,
} from "@/features/workspaces/api/workspaces";
import type { Terminal, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const claude: Terminal = {
  id: "w2",
  name: "Claude",
  agent: "claude",
  status: "idle",
  folder: "/repos/fogde",
  checkout: "/repos/fogde",
};

function workspace(terminals: Terminal[]): Workspace {
  return {
    id: "2",
    name: "fogde",
    folder: "/repos/fogde",
    checkouts: [],
    terminals,
  };
}

function deferred() {
  let resolve: (value: Workspace[]) => void = () => undefined;
  const promise = new Promise<Workspace[]>((done) => {
    resolve = done;
  });

  return { promise, resolve };
}

async function race(
  before: Terminal[],
  update: (queries: QueryClient, machine: Machine) => void,
  after: Terminal[],
) {
  const replies = [deferred(), deferred()];
  const call = vi.fn(() => replies[call.mock.calls.length - 1]?.promise);
  const machine = { id: "local", client: { call } } as unknown as Machine;
  const queries = new QueryClient();
  const query = workspacesQuery(machine);
  queries.setQueryData(query.queryKey, [workspace(before)]);
  const stop = new QueryObserver(queries, query).subscribe(() => undefined);

  update(queries, machine);
  replies[0]?.resolve([workspace(before)]);
  replies[1]?.resolve([workspace(after)]);
  await vi.waitFor(() => {
    expect(queries.isFetching()).toBe(0);
  });

  const terminals = queries.getQueryData(query.queryKey)?.[0]?.terminals;
  stop();
  queries.clear();

  return terminals;
}

test("keeps a new terminal when an older list arrives after it", async () => {
  const terminals = await race(
    [],
    (queries, machine) => {
      addTerminal(queries, machine, "2", claude);
    },
    [claude],
  );

  expect(terminals).toEqual([claude]);
});

test("adds a terminal the list already shows only once", () => {
  const machine = {
    id: "local",
    client: { call: vi.fn() },
  } as unknown as Machine;
  const queries = new QueryClient();
  const { queryKey } = workspacesQuery(machine);
  queries.setQueryData(queryKey, [workspace([claude])]);

  addTerminal(queries, machine, "2", claude);

  expect(queries.getQueryData(queryKey)?.[0]?.terminals).toEqual([claude]);
  queries.clear();
});

test("keeps a closed terminal gone when an older list arrives after it", async () => {
  const terminals = await race(
    [claude],
    (queries, machine) => {
      removeTerminal(queries, machine, claude);
    },
    [],
  );

  expect(terminals).toEqual([]);
});

test("applies a state change to the terminal it names", () => {
  const call = vi.fn();
  const machine = { id: "local", client: { call } } as unknown as Machine;
  const queries = new QueryClient();
  const { queryKey } = workspacesQuery(machine);
  queries.setQueryData(queryKey, [workspace([claude])]);

  const applied = applyState(queries, machine, {
    id: "w2",
    name: "Fix the login",
    agent: "claude",
    status: "blocked",
  });

  expect(applied?.before.status).toBe("idle");
  expect(applied?.workspace.name).toBe("fogde");
  expect(queries.getQueryData(queryKey)?.[0]?.terminals[0]).toEqual({
    ...claude,
    name: "Fix the login",
    status: "blocked",
  });
  queries.clear();
});

test("reloads the workspaces for a terminal it doesn't know", async () => {
  const call = vi.fn(() => Promise.resolve([workspace([claude])]));
  const machine = { id: "local", client: { call } } as unknown as Machine;
  const queries = new QueryClient();
  const query = workspacesQuery(machine);
  queries.setQueryData(query.queryKey, [workspace([])]);
  const stop = new QueryObserver(queries, query).subscribe(() => undefined);

  const applied = applyState(queries, machine, {
    id: "w2",
    name: "Claude",
    agent: "claude",
    status: "working",
  });

  expect(applied).toBeUndefined();
  await vi.waitFor(() => {
    expect(queries.getQueryData(query.queryKey)?.[0]?.terminals).toEqual([
      claude,
    ]);
  });
  stop();
  queries.clear();
});
