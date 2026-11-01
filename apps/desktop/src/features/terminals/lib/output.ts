import type { Terminal } from "@xterm/xterm";

import type { Client } from "@/lib/client";

export function showOutput(
  client: Client,
  id: string,
  terminal: Terminal,
  token: () => number | undefined,
) {
  return client.listen("terminals.output", (output) => {
    if (output.id === id && output.token === token()) {
      terminal.write(decode(output.data));
    }
  });
}

function decode(data: string) {
  return Uint8Array.from(atob(data), (char) => char.charCodeAt(0));
}
