import type { Terminal } from "@xterm/xterm";

import { report } from "@/features/terminals/lib/report";

const SETTLE = 150;

export function copyOnSelect(terminal: Terminal, copied: () => void) {
  let timer = 0;

  const selecting = terminal.onSelectionChange(() => {
    window.clearTimeout(timer);
    if (!terminal.hasSelection()) {
      return;
    }

    timer = window.setTimeout(() => {
      navigator.clipboard
        .writeText(terminal.getSelection())
        .then(copied, (error: unknown) => {
          report("Could not copy", error);
        });
    }, SETTLE);
  });

  return () => {
    window.clearTimeout(timer);
    selecting.dispose();
  };
}
