import { expect, test } from "vitest";

import { watchTerminal } from "@/features/terminals/lib/watch";
import type { Client } from "@/lib/client";

const size = { cols: 80, rows: 24 };

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
