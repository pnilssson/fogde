import { QueryClient, QueryObserver } from "@tanstack/react-query";
import { expect, test, vi } from "vitest";

import {
  addTerminal,
  applyState,
  removeTerminal,
  workspacesQuery,
} from "@/features/workspaces/api/workspaces";
import type { Terminal, Tile, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const claude: Terminal = {
  id: "w2",
  name: "Claude",
  agent: "claude",
  status: "idle",
  folder: "/repos/app",
  branch: "main",
  changes: null,
  checkout: null,
};

function tile(terminal: Terminal): Tile {
  return { kind: "terminal", ...terminal };
}

function workspace(terminals: Terminal[]): Workspace {
  return {
    id: "2",
    name: "app",
    folder: "/repos/app",
    missing: false,
    checkouts: [],
    tiles: terminals.map(tile),
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

  const tiles = queries.getQueryData(query.queryKey)?.[0]?.tiles;
  stop();
  queries.clear();

  return tiles;
}

test("keeps a new terminal when an older list arrives after it", async () => {
  const terminals = await race(
    [],
    (queries, machine) => {
      addTerminal(queries, machine, "2", claude);
    },
    [claude],
  );

  expect(terminals).toEqual([tile(claude)]);
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

  expect(queries.getQueryData(queryKey)?.[0]?.tiles).toEqual([tile(claude)]);
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

test("applies a state change, with its folder and branch, to the terminal it names", () => {
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
    folder: "/repos/app/src",
    branch: "fix-login",
    changes: null,
  });

  expect(applied?.before.status).toBe("idle");
  expect(applied?.workspace.name).toBe("app");
  expect(queries.getQueryData(queryKey)?.[0]?.tiles[0]).toEqual({
    ...tile(claude),
    name: "Fix the login",
    status: "blocked",
    folder: "/repos/app/src",
    branch: "fix-login",
    changes: null,
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
    folder: "/repos/app",
    branch: "main",
    changes: null,
  });

  expect(applied).toBeUndefined();
  await vi.waitFor(() => {
    expect(queries.getQueryData(query.queryKey)?.[0]?.tiles).toEqual([
      tile(claude),
    ]);
  });
  stop();
  queries.clear();
});
