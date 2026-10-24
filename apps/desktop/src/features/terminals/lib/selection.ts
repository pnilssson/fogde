import type { Terminal } from "@xterm/xterm";

import { report } from "@/features/terminals/lib/report";

export function copyOnSelect(
  terminal: Terminal,
  element: HTMLElement,
  copied: () => void,
) {
  let pressed = false;
  let changed = false;

  const selecting = terminal.onSelectionChange(() => {
    changed = terminal.hasSelection();
  });
  const press = () => {
    pressed = true;
  };
  const release = () => {
    if (!pressed) {
      return;
    }

    pressed = false;
    if (!changed) {
      return;
    }

    changed = false;
    navigator.clipboard
      .writeText(terminal.getSelection())
      .then(copied, (error: unknown) => {
        report("Could not copy", error);
      });
  };

  element.addEventListener("mousedown", press, { capture: true });
  document.addEventListener("mouseup", release);

  return () => {
    selecting.dispose();
    element.removeEventListener("mousedown", press, { capture: true });
    document.removeEventListener("mouseup", release);
  };
}
