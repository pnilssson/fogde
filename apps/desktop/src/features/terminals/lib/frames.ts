import type { Terminal } from "@xterm/xterm";

import type { Client } from "@/lib/client";

const reset = "\x1bc";
const nowrap = "\x1b[?7l";

export function showFrames(client: Client, id: string, terminal: Terminal) {
  let open = true;
  let rendering = Promise.resolve();

  const stop = client.listen("terminals.frame", (frame) => {
    if (frame.id !== id) {
      return;
    }

    rendering = rendering.then(
      () =>
        new Promise<void>((resolve) => {
          if (!open) {
            resolve();
            return;
          }

          if (frame.cols !== terminal.cols || frame.rows !== terminal.rows) {
            terminal.resize(frame.cols, frame.rows);
          }
          if (frame.full) {
            terminal.write(reset + nowrap);
          }
          terminal.write(decode(frame.data), resolve);
        }),
    );
  });

  return () => {
    open = false;
    stop();
  };
}

function decode(data: string) {
  return Uint8Array.from(atob(data), (char) => char.charCodeAt(0));
}
