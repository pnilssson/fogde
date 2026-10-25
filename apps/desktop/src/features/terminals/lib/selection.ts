import type { Terminal } from "@xterm/xterm";

import { report } from "@/lib/report";

export function copyOnSelect(
  terminal: Terminal,
  element: HTMLElement,
  copied: () => void,
) {
  let pressed = false;
  let timer = 0;

  const press = () => {
    pressed = true;
  };
  const release = () => {
    if (!pressed) {
      return;
    }

    pressed = false;
    timer = window.setTimeout(() => {
      if (!terminal.hasSelection()) {
        return;
      }

      navigator.clipboard
        .writeText(terminal.getSelection())
        .then(copied, (error: unknown) => {
          report("Could not copy", error);
        });
    });
  };

  element.addEventListener("mousedown", press, { capture: true });
  document.addEventListener("mouseup", release);

  return () => {
    window.clearTimeout(timer);
    element.removeEventListener("mousedown", press, { capture: true });
    document.removeEventListener("mouseup", release);
  };
}
