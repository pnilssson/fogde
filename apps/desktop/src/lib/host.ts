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

const pending = new Map<number, Pending>();
let socket: Promise<WebSocket> | undefined;
let next = 0;

export async function call<M extends Method>(
  method: M,
  ...[params]: Params<M> extends null ? [] : [Params<M>]
): Promise<Result<M>> {
  const open = await connect();
  next += 1;
  const id = next;

  return new Promise((resolve, reject) => {
    pending.set(id, {
      resolve: (value) => {
        resolve(value as Result<M>);
      },
      reject,
    });
    open.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
  });
}

// The native pickers need the desktop app on the host's own computer.
export async function local(): Promise<boolean> {
  return isTauri() && (await call("client.local"));
}

function connect(): Promise<WebSocket> {
  socket ??= open().catch((error: unknown) => {
    socket = undefined;
    throw error;
  });

  return socket;
}

async function open(): Promise<WebSocket> {
  const connection = await invoke<Connection>("connection");
  const ws = new WebSocket(`${connection.url}?token=${connection.token}`);

  ws.addEventListener("message", (event) => {
    receive(event.data as string);
  });
  ws.addEventListener("close", () => {
    socket = undefined;
    for (const waiting of pending.values()) {
      waiting.reject(new Error("Lost the connection to Fogde."));
    }
    pending.clear();
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
        reject(new Error("Could not connect to Fogde."));
      },
      { once: true },
    );
  });

  return ws;
}

function receive(data: string) {
  const reply = JSON.parse(data) as Reply;
  if (reply.id === null) {
    return;
  }

  const waiting = pending.get(reply.id);
  if (!waiting) {
    return;
  }

  pending.delete(reply.id);
  if (reply.error) {
    waiting.reject(new Error(reply.error.message));
    return;
  }

  waiting.resolve(reply.result);
}
