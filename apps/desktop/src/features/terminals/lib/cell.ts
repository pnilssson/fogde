import type { Terminal } from "@xterm/xterm";

export function cell(offset: number, extent: number, count: number) {
  return Math.min(
    count - 1,
    Math.max(0, Math.floor((offset / extent) * count)),
  );
}

export function locate(
  event: MouseEvent,
  element: HTMLElement,
  terminal: Terminal,
) {
  const screen = element.querySelector(".xterm-screen") ?? element;
  const rect = screen.getBoundingClientRect();
  const point = {
    column: cell(event.clientX - rect.left, rect.width, terminal.cols),
    row: cell(event.clientY - rect.top, rect.height, terminal.rows),
  };
  if (event.clientY < rect.top) {
    return { point, edge: "up" as const };
  }
  if (event.clientY > rect.bottom) {
    return { point, edge: "down" as const };
  }

  return { point, edge: undefined };
}
