import type { WatchTerminalParams } from "@/generated/protocol";
import type { Client } from "@/lib/client";

const operations = new WeakMap<Client, Map<string, Promise<void>>>();
let tokens = 0;

function queue(client: Client) {
  let queued = operations.get(client);
  if (!queued) {
    queued = new Map();
    operations.set(client, queued);
  }

  return queued;
}

function enqueue(client: Client, id: string, step: () => Promise<void>) {
  const queued = queue(client);
  const next = (queued.get(id) ?? Promise.resolve())
    .then(step)
    .catch(() => undefined);
  queued.set(id, next);
  void next.then(() => {
    if (queued.get(id) === next) {
      queued.delete(id);
    }
  });
}

export function watchTerminal(
  client: Client,
  id: string,
  view: () => Omit<WatchTerminalParams, "id" | "token">,
  ready: () => void,
  failed: (error: unknown) => void,
  begin: (token: number) => void = () => undefined,
): (release?: boolean) => void {
  const lifetime = new AbortController();
  const active = () => !lifetime.signal.aborted;
  let attached = false;

  enqueue(client, id, async () => {
    if (!active()) {
      return;
    }

    try {
      const token = ++tokens;
      begin(token);
      await client.call("terminals.watch", { id, ...view(), token });
      attached = true;
      if (active()) {
        ready();
      }
    } catch (error) {
      if (active()) {
        failed(error);
      }
    }
  });

  return (release = true) => {
    if (!active()) {
      return;
    }
    lifetime.abort();
    enqueue(client, id, async () => {
      if (attached && release && client.getSnapshot().state === "connected") {
        await client.call("terminals.unwatch", id).catch(() => undefined);
      }
    });
  };
}
