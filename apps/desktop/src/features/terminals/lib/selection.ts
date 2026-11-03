import type { Terminal } from "@xterm/xterm";

import { copyFromPrograms } from "@/features/terminals/lib/copies";
import { report } from "@/lib/report";

export interface Selection {
  has: () => boolean;
  copy: () => void;
  clear: () => void;
  dispose: () => void;
}

export function handleSelection(
  terminal: Terminal,
  element: HTMLElement,
  copied: () => void,
): Selection {
  let pressed = false;
  let timer = 0;

  const copy = () => {
    const text = terminal.getSelection();
    if (!text) {
      return;
    }

    navigator.clipboard.writeText(text).then(copied, (error: unknown) => {
      report("Could not copy", error);
    });
  };

  const press = (event: MouseEvent) => {
    pressed = event.button === 0;
  };

  const release = (event: MouseEvent) => {
    if (!pressed || event.button !== 0) {
      return;
    }

    pressed = false;
    timer = window.setTimeout(() => {
      if (terminal.hasSelection()) {
        copy();
      }
    });
  };

  const typing = terminal.onKey(() => {
    terminal.clearSelection();
  });
  const programs = copyFromPrograms(terminal, copied);
  element.addEventListener("mousedown", press);
  document.addEventListener("mouseup", release);

  return {
    has: () => terminal.hasSelection(),
    copy,
    clear: () => {
      terminal.clearSelection();
    },
    dispose: () => {
      window.clearTimeout(timer);
      typing.dispose();
      programs.dispose();
      element.removeEventListener("mousedown", press);
      document.removeEventListener("mouseup", release);
    },
  };
}
