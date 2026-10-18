import { invoke, isTauri } from "@tauri-apps/api/core";

import type { Connection, Methods } from "@/generated/protocol";

type Method = keyof Methods;
type Params<M extends Method> = Methods[M]["params"];
type Result<M extends Method> = Methods[M]["result"];

interface Reply {
  id: number | null;
  result?: unknown;
  error?: { code: number; message: string };
}

interface Pending {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

interface Channel {
  ws: WebSocket;
  pending: Map<number, Pending>;
}

type Link = "own" | "chosen";

const channels = new Map<Link, Promise<Channel>>();
let next = 0;

export function call<M extends Method>(
  method: M,
  ...[params]: Params<M> extends null ? [] : [Params<M>]
): Promise<Result<M>> {
  return send("chosen", method, params);
}

export function callOwn<M extends Method>(
  method: M,
  ...[params]: Params<M> extends null ? [] : [Params<M>]
): Promise<Result<M>> {
  return send("own", method, params);
}

export function reconnect() {
  const channel = channels.get("chosen");
  channels.delete("chosen");
  void channel?.then(({ ws }) => {
    ws.close();
  });
}

export async function local(): Promise<boolean> {
  return isTauri() && (await call("client.local"));
}

async function send<M extends Method>(
  link: Link,
  method: M,
  params: Params<M> | undefined,
): Promise<Result<M>> {
  const channel = await connect(link);
  next += 1;
  const id = next;

  return new Promise((resolve, reject) => {
    channel.pending.set(id, {
      resolve: (value) => {
        resolve(value as Result<M>);
      },
      reject,
    });
    channel.ws.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
  });
}

function connect(link: Link): Promise<Channel> {
  const existing = channels.get(link);
  if (existing) {
    return existing;
  }

  const opening: Promise<Channel> = open(link, () => {
    forget(link, opening);
  }).catch((error: unknown) => {
    forget(link, opening);
    throw error;
  });
  channels.set(link, opening);

  return opening;
}

function forget(link: Link, opening: Promise<Channel>) {
  if (channels.get(link) === opening) {
    channels.delete(link);
  }
}

async function open(link: Link, onClose: () => void): Promise<Channel> {
  const connection = await invoke<Connection>("connection");
  const server = link === "chosen" && (await callOwn("server.get")) !== null;
  const ws = new WebSocket(
    `${connection.url}${server ? "server" : ""}?token=${connection.token}`,
  );
  const channel: Channel = { ws, pending: new Map() };

  ws.addEventListener("message", (event) => {
    receive(channel, event.data as string);
  });
  ws.addEventListener("close", () => {
    onClose();
    for (const waiting of channel.pending.values()) {
      waiting.reject(
        new Error(
          server
            ? "Lost the connection to your server."
            : "Lost the connection to Fogde.",
        ),
      );
    }
    channel.pending.clear();
  });

  await new Promise<void>((resolve, reject) => {
    ws.addEventListener(
      "open",
      () => {
        resolve();
      },
      { once: true },
    );
    ws.addEventListener(
      "error",
      () => {
        reject(
          new Error(
            server
              ? "Could not reach your server."
              : "Could not connect to Fogde.",
          ),
        );
      },
      { once: true },
    );
  });

  return channel;
}

function receive(channel: Channel, data: string) {
  const reply = JSON.parse(data) as Reply;
  if (reply.id === null) {
    return;
  }

  const waiting = channel.pending.get(reply.id);
  if (!waiting) {
    return;
  }

  channel.pending.delete(reply.id);
  if (reply.error) {
    waiting.reject(new Error(reply.error.message));
    return;
  }

  waiting.resolve(reply.result);
}
