import { expect, test } from "vitest";

import { watchTerminal } from "@/features/terminals/lib/watch";
import type { Client } from "@/lib/client";

const size = () => ({ cols: 80, rows: 24 });

function fake(call: (method: string) => Promise<unknown>, state = "connected") {
  return { call, getSnapshot: () => ({ state }) } as unknown as Client;
}

function fail(error: unknown) {
  throw error;
}

test("a stalled server watch does not block a local terminal", async () => {
  const remote = fake(() => new Promise(() => undefined), "connecting");
  const local = fake(() => Promise.resolve());
  watchTerminal(remote, "w1", size, () => undefined, fail);

  const ready = new Promise<"ready">((resolve) => {
    watchTerminal(
      local,
      "w1",
      size,
      () => {
        resolve("ready");
      },
      fail,
    );
  });
  const blocked = new Promise<"blocked">((resolve) => {
    setTimeout(() => {
      resolve("blocked");
    }, 50);
  });

  expect(await Promise.race([ready, blocked])).toBe("ready");
});

test("a late cleanup finishes before the next watch on the same machine", async () => {
  const calls: string[] = [];
  let finish: () => void = () => undefined;
  const client = fake((method) => {
    calls.push(method);
    if (calls.length > 1) {
      return Promise.resolve();
    }

    return new Promise<void>((resolve) => {
      finish = resolve;
    });
  });
  let stale = false;
  const stop = watchTerminal(
    client,
    "w1",
    size,
    () => {
      stale = true;
    },
    fail,
  );
  await new Promise((resolve) => setTimeout(resolve));

  stop();
  const ready = new Promise<void>((resolve) => {
    watchTerminal(client, "w1", size, resolve, fail);
  });
  finish();
  await ready;

  expect(stale).toBe(false);
  expect(calls).toEqual([
    "terminals.watch",
    "terminals.unwatch",
    "terminals.watch",
  ]);
});

test("a slow watch on one terminal does not hold back another on the same machine", async () => {
  const client = {
    call: (_: string, params: { id: string }) =>
      params.id === "t1" ? new Promise(() => undefined) : Promise.resolve(),
    getSnapshot: () => ({ state: "connected" }),
  } as unknown as Client;
  watchTerminal(client, "t1", size, () => undefined, fail);

  const ready = new Promise<"ready">((resolve) => {
    watchTerminal(
      client,
      "t2",
      size,
      () => {
        resolve("ready");
      },
      fail,
    );
  });
  const blocked = new Promise<"blocked">((resolve) => {
    setTimeout(() => {
      resolve("blocked");
    }, 50);
  });

  expect(await Promise.race([ready, blocked])).toBe("ready");
});

test("a new watch starts taking output only after the old one let go", async () => {
  const calls: string[] = [];
  const client = fake((method) => {
    calls.push(method);
    return Promise.resolve();
  });
  let stop: () => void = () => undefined;
  await new Promise<void>((resolve) => {
    stop = watchTerminal(client, "w1", size, resolve, fail, () =>
      calls.push("begin"),
    );
  });

  stop();
  await new Promise<void>((resolve) => {
    watchTerminal(client, "w1", size, resolve, fail, () => calls.push("begin"));
  });

  expect(calls).toEqual([
    "begin",
    "terminals.watch",
    "terminals.unwatch",
    "begin",
    "terminals.watch",
  ]);
});

test("each watch carries a new token that begin sees first", async () => {
  const sent: unknown[] = [];
  const client = {
    call: (_: string, params: unknown) => {
      sent.push(params);
      return Promise.resolve();
    },
    getSnapshot: () => ({ state: "connected" }),
  } as unknown as Client;
  const begun: number[] = [];

  const stop = await new Promise<() => void>((resolve) => {
    const stopping = watchTerminal(
      client,
      "w1",
      size,
      () => {
        resolve(stopping);
      },
      fail,
      (token) => begun.push(token),
    );
  });
  stop();
  await new Promise<void>((resolve) => {
    watchTerminal(client, "w1", size, resolve, fail, (token) =>
      begun.push(token),
    );
  });

  const [first, second] = begun;
  expect(first).not.toBe(second);
  expect(sent).toContainEqual({ id: "w1", size: size(), token: first });
  expect(sent).toContainEqual({ id: "w1", size: size(), token: second });
});

test("a watch whose start throws reports it and lets the next one run", async () => {
  const client = fake(() => Promise.resolve());
  const failed = await new Promise<unknown>((resolve) => {
    watchTerminal(
      client,
      "w1",
      size,
      () => undefined,
      resolve,
      () => {
        throw new Error("no terminal");
      },
    );
  });

  await new Promise<void>((resolve) => {
    watchTerminal(client, "w1", size, resolve, fail);
  });

  expect(failed).toBeInstanceOf(Error);
});
