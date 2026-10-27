import type { Terminal } from "@xterm/xterm";

import { cell } from "@/features/terminals/lib/cell";

interface Position {
  column: number;
  row: number;
}

export function clickThrough(
  terminal: Terminal,
  element: HTMLElement,
  send: (position: Position) => void,
) {
  let pressed: Position | undefined;

  const locate = (event: MouseEvent) => {
    const screen = element.querySelector(".xterm-screen") ?? element;
    const rect = screen.getBoundingClientRect();

    return {
      column: cell(event.clientX - rect.left, rect.width, terminal.cols),
      row: cell(event.clientY - rect.top, rect.height, terminal.rows),
    };
  };
  const press = (event: MouseEvent) => {
    const plain =
      event.button === 0 &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      !event.shiftKey;
    pressed = plain ? locate(event) : undefined;
  };
  const release = (event: MouseEvent) => {
    const start = pressed;
    pressed = undefined;
    if (!start || event.button !== 0 || terminal.hasSelection()) {
      return;
    }

    const end = locate(event);
    if (end.column !== start.column || end.row !== start.row) {
      return;
    }

    send(end);
  };

  element.addEventListener("mousedown", press, { capture: true });
  document.addEventListener("mouseup", release);

  return () => {
    element.removeEventListener("mousedown", press, { capture: true });
    document.removeEventListener("mouseup", release);
  };
}
