import type { Client } from "@/lib/client";

const interval = 15_000;
const deadline = 10_000;
const first = 1_000;
const ceiling = 30_000;

type Watched = Pick<
  Client,
  "buffered" | "call" | "getSnapshot" | "lose" | "reconnect" | "subscribe"
>;

export function keepAlive<C extends Watched>(client: C) {
  let state = client.getSnapshot().state;
  let connection = 0;
  let attempt = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  const later = (delay: number, run: () => void) => {
    clearTimeout(timer);
    timer = setTimeout(run, delay);
  };

  const check = async (current: number) => {
    const ping = client.call("health.ping").then(
      () => true,
      () => true,
    );
    let answered: boolean;
    let left = client.buffered();
    for (;;) {
      answered = await Promise.race([
        ping,
        new Promise<false>((resolve) => {
          setTimeout(() => {
            resolve(false);
          }, deadline);
        }),
      ]);
      const before = left;
      left = client.buffered();
      if (answered || before === 0 || left >= before) {
        break;
      }
    }
    if (current !== connection || state !== "connected") {
      return;
    }

    if (answered) {
      later(interval, () => void check(current));
    } else {
      client.lose("The connection stopped responding.");
    }
  };

  client.subscribe(() => {
    const snapshot = client.getSnapshot();
    if (snapshot.state === state) {
      return;
    }

    state = snapshot.state;
    clearTimeout(timer);
    if (state === "connected") {
      attempt = 0;
      connection += 1;
      const current = connection;
      later(interval, () => void check(current));
    } else if (state === "disconnected" && snapshot.retryable) {
      const delay = Math.min(ceiling, first * 2 ** attempt);
      attempt += 1;
      later(delay * (0.5 + Math.random() / 2), () => {
        void client.reconnect().catch(() => undefined);
      });
    }
  });

  return client;
}
