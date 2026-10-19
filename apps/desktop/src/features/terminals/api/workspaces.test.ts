import { QueryClient, QueryObserver } from "@tanstack/react-query";
import { expect, test, vi } from "vitest";

import {
  addTerminal,
  removeTerminal,
  workspacesQuery,
} from "@/features/terminals/api/workspaces";
import type { Terminal, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const claude: Terminal = {
  id: "w2",
  name: "fogde · Claude",
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
