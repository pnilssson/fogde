import type { Terminal } from "@xterm/xterm";

import { cell } from "@/features/terminals/lib/cell";
import type { ScrollTerminalParams } from "@/generated/protocol";

export type Scroll = Omit<ScrollTerminalParams, "id">;

export function scrollOnWheel(
  terminal: Terminal,
  element: HTMLElement,
  send: (scroll: Scroll) => void,
) {
  let pending = 0;

  terminal.attachCustomWheelEventHandler((event) => {
    event.preventDefault();

    const rect = element.getBoundingClientRect();
    const height = rect.height / terminal.rows;
    pending += pixels(event, height, terminal.rows);
    const lines = Math.trunc(pending / height);
    if (lines === 0) {
      return false;
    }

    pending -= lines * height;
    send({
      direction: lines < 0 ? "up" : "down",
      lines: Math.abs(lines),
      column: cell(event.clientX - rect.left, rect.width, terminal.cols),
      row: cell(event.clientY - rect.top, rect.height, terminal.rows),
    });

    return false;
  });
}

function pixels(event: WheelEvent, height: number, rows: number) {
  if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
    return event.deltaY * height;
  }
  if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
    return event.deltaY * height * rows;
  }

  return event.deltaY;
}
