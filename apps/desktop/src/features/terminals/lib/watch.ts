import type { Size } from "@/generated/protocol";
import type { Client } from "@/lib/client";

const operations = new WeakMap<Client, Promise<void>>();

export function watchTerminal(
  client: Client,
  id: string,
  size: Size,
  ready: () => void,
  failed: (error: unknown) => void,
): (release?: boolean) => void {
  const lifetime = new AbortController();
  const active = () => !lifetime.signal.aborted;
  let attached = false;

  const opening = (operations.get(client) ?? Promise.resolve()).then(
    async () => {
      if (!active()) {
        return;
      }

      try {
        await client.call("terminals.watch", { id, size });
        attached = true;
        if (active()) {
          ready();
        }
      } catch (error) {
        if (active()) {
          failed(error);
        }
      }
    },
  );
  operations.set(client, opening);

  return (release = true) => {
    if (!active()) {
      return;
    }
    lifetime.abort();
    const closing = (operations.get(client) ?? Promise.resolve())
      .then(async () => {
        if (attached && release && client.getSnapshot().state === "connected") {
          await client.call("terminals.unwatch", id);
        }
      })
      .catch(() => undefined);
    operations.set(client, closing);
  };
}
