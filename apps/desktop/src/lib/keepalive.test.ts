import { afterEach, beforeEach, expect, test, vi } from "vitest";

import type { ConnectionState } from "@/lib/client";
import { keepAlive } from "@/lib/keepalive";

class Fake {
  snapshot: ConnectionState = { state: "idle", error: null, retryable: false };
  answer = true;
  pings = 0;
  reconnects = 0;
  lost: string[] = [];
  queued: number[] = [];
  private changes = new Set<() => void>();

  getSnapshot = () => this.snapshot;

  buffered = () => this.queued.shift() ?? 0;

  subscribe = (listener: () => void) => {
    this.changes.add(listener);

    return () => {
      this.changes.delete(listener);
    };
  };

  set(state: ConnectionState["state"], retryable = false) {
    this.snapshot = { state, error: null, retryable };
    for (const listener of this.changes) {
      listener();
    }
  }

  call = () => {
    this.pings += 1;

    return this.answer
      ? Promise.resolve(null)
      : new Promise<null>(() => undefined);
  };

  reconnect = () => {
    this.reconnects += 1;

    return Promise.resolve();
  };

  lose = (error: string) => {
    this.lost.push(error);
    this.set("disconnected", true);
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.spyOn(Math, "random").mockReturnValue(1);
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

test("pings a connected host every 15 seconds", async () => {
  const client = keepAlive(new Fake());
  client.set("connected");

  await vi.advanceTimersByTimeAsync(14_999);
  expect(client.pings).toBe(0);
  await vi.advanceTimersByTimeAsync(1);
  expect(client.pings).toBe(1);
  await vi.advanceTimersByTimeAsync(15_000);
  expect(client.pings).toBe(2);
  expect(client.lost).toEqual([]);
});

test("drops a connection that stops answering and reconnects", async () => {
  const client = keepAlive(new Fake());
  client.set("connected");
  client.answer = false;

  await vi.advanceTimersByTimeAsync(15_000 + 9_999);
  expect(client.lost).toEqual([]);
  await vi.advanceTimersByTimeAsync(1);
  expect(client.lost).toEqual(["The connection stopped responding."]);

  await vi.advanceTimersByTimeAsync(1_000);
  expect(client.reconnects).toBe(1);
});

test("waits for a large upload to go out before giving up on the ping", async () => {
  const client = keepAlive(new Fake());
  client.set("connected");
  client.answer = false;
  client.queued = [9_000_000, 6_000_000, 2_000_000, 0];

  await vi.advanceTimersByTimeAsync(15_000 + 30_000);
  expect(client.lost).toEqual([]);
  await vi.advanceTimersByTimeAsync(10_000);
  expect(client.lost).toEqual(["The connection stopped responding."]);
});

test("drops a connection whose upload stops moving", async () => {
  const client = keepAlive(new Fake());
  client.set("connected");
  client.answer = false;
  client.queued = [9_000_000, 9_000_000];

  await vi.advanceTimersByTimeAsync(15_000 + 10_000);
  expect(client.lost).toEqual(["The connection stopped responding."]);
});

test("retries a lost connection with growing delays up to 30 seconds", async () => {
  const client = keepAlive(new Fake());
  const waits: number[] = [];

  for (let attempt = 0; attempt < 7; attempt += 1) {
    client.set("disconnected", true);
    let waited = 0;
    while (client.reconnects === attempt) {
      await vi.advanceTimersByTimeAsync(1_000);
      waited += 1_000;
    }
    waits.push(waited);
    client.set("connecting");
  }

  expect(waits).toEqual([1_000, 2_000, 4_000, 8_000, 16_000, 30_000, 30_000]);
});

test("starts over after a successful connection", async () => {
  const client = keepAlive(new Fake());
  for (let attempt = 0; attempt < 3; attempt += 1) {
    client.set("disconnected", true);
    await vi.advanceTimersByTimeAsync(30_000);
    client.set("connecting");
  }
  client.set("connected");
  client.set("disconnected", true);

  const before = client.reconnects;
  await vi.advanceTimersByTimeAsync(1_000);
  expect(client.reconnects).toBe(before + 1);
});

test("never retries a failure the user must fix, or a reset connection", async () => {
  const client = keepAlive(new Fake());
  client.set("disconnected", false);
  await vi.advanceTimersByTimeAsync(60_000);
  expect(client.reconnects).toBe(0);

  client.set("connected");
  client.set("idle");
  await vi.advanceTimersByTimeAsync(60_000);
  expect(client.pings).toBe(0);
  expect(client.reconnects).toBe(0);
});
