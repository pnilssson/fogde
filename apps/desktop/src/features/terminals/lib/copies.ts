import type { Terminal } from "@xterm/xterm";

import { report } from "@/lib/report";

export function copyFromPrograms(terminal: Terminal, copied: () => void) {
  return terminal.parser.registerOscHandler(52, (data) => {
    const encoded = data.slice(data.indexOf(";") + 1);
    if (!data.includes(";") || encoded === "" || encoded === "?") {
      return true;
    }

    let text: string;
    try {
      const bytes = Uint8Array.from(atob(encoded), (char) =>
        char.charCodeAt(0),
      );
      text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    } catch {
      return true;
    }

    navigator.clipboard.writeText(text).then(copied, (error: unknown) => {
      report("Could not copy", error);
    });

    return true;
  });
}
