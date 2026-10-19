import {
  offline,
  type Connection,
  type Events,
  type Methods,
} from "@/generated/protocol";

export type Method = keyof Methods;
export type Args<M extends Method> = Methods[M]["params"] extends null
  ? []
  : [Methods[M]["params"]];
export type Result<M extends Method> = Methods[M]["result"];

export interface ConnectionState {
  state: "idle" | "connecting" | "connected" | "disconnected";
  error: string | null;
  retryable: boolean;
}

interface Pending {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

interface Channel {
  ws: WebSocket;
  pending: Map<number, Pending>;
}

interface Incoming {
  id?: number | null;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code: number; message: string };
}

type Listener = (params: unknown) => void;

export class ConnectionError extends Error {}

export class RpcError extends Error {
  readonly code: number;

  constructor(message: string, code: number) {
    super(message);
    this.code = code;
  }
}

function retryable(error: unknown) {
  return !(error instanceof RpcError) || error.code === offline;
}

export class Client {
  private opening?: Promise<Channel>;
  private socket?: WebSocket;
  private interrupt?: () => void;
  private generation = 0;
  private next = 0;
  private snapshot: ConnectionState = {
    state: "idle",
    error: null,
    retryable: false,
  };
  private changes = new Set<() => void>();
  private listeners = new Map<string, Set<Listener>>();
  private address: () => Promise<Connection>;

  constructor(address: () => Promise<Connection>) {
    this.address = address;
  }

  getSnapshot = () => this.snapshot;

  subscribe = (listener: () => void) => {
    this.changes.add(listener);

    return () => {
      this.changes.delete(listener);
    };
  };

  listen<E extends keyof Events>(
    event: E,
    listener: (params: Events[E]) => void,
  ) {
    const listening = this.listeners.get(event) ?? new Set<Listener>();
    this.listeners.set(event, listening);
    listening.add(listener as Listener);

    return () => {
      listening.delete(listener as Listener);
    };
  }

  reset() {
    this.generation += 1;
    this.opening = undefined;
    const socket = this.socket;
    this.socket = undefined;
    const interrupt = this.interrupt;
    this.interrupt = undefined;
    interrupt?.();
    socket?.close();
    this.update("idle");
  }

  async reconnect() {
    this.reset();
    await this.connect();
  }

  async call<M extends Method>(
    method: M,
    ...[params]: Args<M>
  ): Promise<Result<M>> {
    const channel = await this.connect();
    if (channel.ws.readyState !== WebSocket.OPEN) {
      throw new ConnectionError("The connection closed.");
    }

    this.next += 1;
    const id = this.next;

    return new Promise((resolve, reject) => {
      channel.pending.set(id, {
        resolve: (value) => {
          resolve(value as Result<M>);
        },
        reject,
      });
      try {
        channel.ws.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
      } catch (error) {
        channel.pending.delete(id);
        reject(new ConnectionError("The connection closed.", { cause: error }));
      }
    });
  }

  private update(
    state: ConnectionState["state"],
    error: string | null = null,
    retry = false,
  ) {
    this.snapshot = { state, error, retryable: retry };
    for (const listener of this.changes) {
      listener();
    }
  }

  private connect(): Promise<Channel> {
    if (this.opening) {
      return this.opening;
    }

    const generation = this.generation;
    this.update("connecting");
    this.opening = this.open(generation).catch((error: unknown) => {
      if (generation === this.generation) {
        this.opening = undefined;
        const socket = this.socket;
        this.socket = undefined;
        this.interrupt = undefined;
        socket?.close();
        this.update(
          "disconnected",
          error instanceof Error ? error.message : String(error),
          retryable(error),
        );
      }
      throw error;
    });

    return this.opening;
  }

  private async open(generation: number): Promise<Channel> {
    const connection = await this.address();
    if (generation !== this.generation) {
      throw new ConnectionError("The connection was replaced.");
    }

    const ws = new WebSocket(`${connection.url}?token=${connection.token}`);
    this.socket = ws;
    const channel: Channel = { ws, pending: new Map() };
    let cancelOpening: () => void = () => undefined;
    const rejectPending = () => {
      for (const waiting of channel.pending.values()) {
        waiting.reject(new ConnectionError("The connection was lost."));
      }
      channel.pending.clear();
    };
    this.interrupt = () => {
      rejectPending();
      cancelOpening();
    };
    ws.addEventListener("message", (event) => {
      this.receive(channel, event.data as string);
    });
    ws.addEventListener("close", () => {
      rejectPending();
      if (this.socket === ws) {
        this.socket = undefined;
        this.opening = undefined;
        this.interrupt = undefined;
        this.update("disconnected", "The connection was lost.", true);
      }
    });

    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new ConnectionError("The connection timed out."));
        ws.close();
      }, 30_000);
      const finish = (error?: Error) => {
        clearTimeout(timer);
        if (error) {
          reject(error);
        } else {
          resolve();
        }
      };
      cancelOpening = () => {
        finish(new ConnectionError("The connection was replaced."));
      };
      ws.addEventListener(
        "open",
        () => {
          finish();
        },
        { once: true },
      );
      ws.addEventListener(
        "close",
        () => {
          finish(new ConnectionError("The connection closed."));
        },
        { once: true },
      );
      ws.addEventListener(
        "error",
        () => {
          finish(new ConnectionError("Could not connect."));
        },
        { once: true },
      );
    });
    if (generation !== this.generation) {
      ws.close();
      throw new ConnectionError("The connection was replaced.");
    }
    this.update("connected");

    return channel;
  }

  private receive(channel: Channel, data: string) {
    if (this.socket !== channel.ws) {
      return;
    }
    const reply = JSON.parse(data) as Incoming;
    if (reply.method !== undefined) {
      for (const listener of this.listeners.get(reply.method) ?? []) {
        listener(reply.params);
      }
      return;
    }
    if (reply.id === undefined || reply.id === null) {
      return;
    }

    const waiting = channel.pending.get(reply.id);
    if (!waiting) {
      return;
    }
    channel.pending.delete(reply.id);
    if (reply.error) {
      waiting.reject(new RpcError(reply.error.message, reply.error.code));
    } else {
      waiting.resolve(reply.result);
    }
  }
}
