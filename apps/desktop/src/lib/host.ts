import { invoke } from "@tauri-apps/api/core";

import type { Connection } from "@/generated/protocol";
import { Client, type Args, type Method, type Result } from "@/lib/client";

const connection = () => invoke<Connection>("connection");

export const own = new Client(connection);

export function callOwn<M extends Method>(
  method: M,
  ...params: Args<M>
): Promise<Result<M>> {
  return own.call(method, ...params);
}

export function serverClient(id: string) {
  return new Client(async () => {
    await callOwn("server.connect", id);
    const address = await connection();

    return {
      ...address,
      url: `${address.url}servers/${encodeURIComponent(id)}`,
    };
  });
}
