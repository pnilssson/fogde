import { afterEach, expect, test, vi } from "vitest";

import { offline } from "@/generated/protocol";
import { Client, RpcError } from "@/lib/client";

interface Options {
  failures: number;
  stall: boolean;
  error: string | null;
  hold: boolean;
  deferClose: boolean;
}

interface Request {
  id: number;
  method: string;
}

const sockets: Socket[] = [];
const requests: { remote: boolean; method: string }[] = [];
const options: Options = {
  failures: 0,
  stall: false,
  error: null,
  hold: false,
  deferClose: false,
};

class Socket extends EventTarget {
  static OPEN = 1;
  readyState = 0;
  remote: boolean;

  constructor(url: string) {
    super();
    this.remote = url.includes("/server?");
    sockets.push(this);
    queueMicrotask(() => {
      if (this.remote && options.stall) {
        return;
      }

      if (this.remote && options.failures > 0) {
        options.failures -= 1;
        this.dispatchEvent(new Event("error"));
        this.close();
        return;
      }

      this.readyState = Socket.OPEN;
      this.dispatchEvent(new Event("open"));
    });
  }

  message(reply: object) {
    this.dispatchEvent(
      new MessageEvent("message", {
        data: JSON.stringify({ jsonrpc: "2.0", ...reply }),
      }),
    );
  }

  send(data: string) {
    const { id, method } = JSON.parse(data) as Request;
    requests.push({ remote: this.remote, method });
    if (this.remote && options.hold && method === "terminals.watch") {
      return;
    }

    queueMicrotask(() => {
      this.message(
        this.remote && options.error
          ? { id, error: { message: options.error } }
          : {
              id,
              result: [
                {
                  id: "w1",
                  name: this.remote ? "Remote" : "Local",
                  terminals: [],
                },
              ],
            },
      );
    });
  }

  close() {
    if (this.remote && options.deferClose) {
      this.readyState = 2;
      return;
    }

    this.readyState = 3;
    this.dispatchEvent(new Event("close"));
  }
}

vi.stubGlobal("WebSocket", Socket);

const local = new Client(() =>
  Promise.resolve({ url: "ws://test/", token: "test" }),
);
const remote = new Client(() =>
  Promise.resolve({ url: "ws://test/server", token: "test" }),
);

afterEach(() => {
  local.reset();
  remote.reset();
  for (const socket of sockets) {
    socket.dispatchEvent(new Event("close"));
  }
  sockets.length = 0;
  requests.length = 0;
  Object.assign(options, {
    failures: 0,
    stall: false,
    error: null,
    hold: false,
    deferClose: false,
  });
});

function within<T>(promise: Promise<T>, ms = 50) {
  return Promise.race([
    promise,
    new Promise<"blocked">((resolve) => {
      setTimeout(() => {
        resolve("blocked");
      }, ms);
    }),
  ]);
}

test("an unavailable server settles without retrying or blocking this computer", async () => {
  options.failures = 1;

  await expect(remote.call("workspaces.list")).rejects.toThrow(
    /Could not connect/,
  );
  expect(remote.getSnapshot().state).toBe("disconnected");
  expect((await local.call("workspaces.list"))[0]?.name).toBe("Local");
  expect(sockets.filter((socket) => socket.remote)).toHaveLength(1);
});

test("a pending server handshake does not block a local request", async () => {
  options.stall = true;
  void remote.call("workspaces.list").catch(() => undefined);

  expect((await local.call("workspaces.list"))[0]?.name).toBe("Local");
  expect(remote.getSnapshot().state).toBe("connecting");
  expect(local.getSnapshot().state).toBe("connected");
});

test("reconnect replaces only the disconnected machine's socket", async () => {
  options.failures = 1;
  await local.call("workspaces.list");
  const socket = sockets[0];
  await expect(remote.call("workspaces.list")).rejects.toThrow();

  await remote.reconnect();

  expect((await remote.call("workspaces.list"))[0]?.name).toBe("Remote");
  expect(socket?.readyState).toBe(Socket.OPEN);
  expect(sockets.filter((socket) => !socket.remote)).toHaveLength(1);
});

test("events never cross from one machine's connection to another", async () => {
  const frames = { local: 0, remote: 0 };
  local.listen("terminals.frame", () => {
    frames.local += 1;
  });
  remote.listen("terminals.frame", () => {
    frames.remote += 1;
  });
  await local.call("workspaces.list");
  await remote.call("workspaces.list");
  const socket = sockets.find((socket) => socket.remote);
  const frame = {
    method: "terminals.frame",
    params: { id: "w1", cols: 80, rows: 24, full: true, data: "aGk=" },
  };

  socket?.message(frame);
  remote.reset();
  socket?.message(frame);

  expect(frames).toEqual({ local: 0, remote: 1 });
});

test("a host error is reported without marking the machine offline", async () => {
  options.error = "could not start the terminal service";

  await expect(remote.call("workspaces.list")).rejects.toThrow(
    /could not start the terminal service/,
  );
  expect(remote.getSnapshot().state).toBe("connected");
  expect(requests).toHaveLength(1);
});

test("reconnect rejects old requests before the old socket finishes closing", async () => {
  await remote.call("workspaces.list");
  options.hold = true;
  options.deferClose = true;
  const outcome = remote
    .call("terminals.watch", { id: "w1", size: { cols: 80, rows: 24 } })
    .then(
      () => "resolved",
      () => "rejected",
    );

  await remote.reconnect();

  expect(await within(outcome)).toBe("rejected");
});

test("a server that cannot be reached is retried when the user returns", async () => {
  const unreachable = new Client(() =>
    Promise.reject(new RpcError("could not reach my-vps", offline)),
  );

  await expect(unreachable.call("workspaces.list")).rejects.toThrow(
    /could not reach my-vps/,
  );
  expect(unreachable.getSnapshot()).toEqual({
    state: "disconnected",
    error: "could not reach my-vps",
    retryable: true,
  });
});

test("a rejected key waits for the user instead of retrying", async () => {
  const rejected = new Client(() =>
    Promise.reject(new RpcError("my-vps did not accept your SSH key", -32000)),
  );

  await expect(rejected.call("workspaces.list")).rejects.toThrow();
  expect(rejected.getSnapshot().retryable).toBe(false);
});

test("a lost connection is retried when the user returns", async () => {
  await remote.call("workspaces.list");

  sockets.find((socket) => socket.remote)?.close();

  expect(remote.getSnapshot()).toEqual({
    state: "disconnected",
    error: "The connection was lost.",
    retryable: true,
  });
});
