import type { Terminal } from "@xterm/xterm";

import type { Size } from "@/generated/protocol";

interface Area {
  width: number;
  height: number;
}

interface Internals {
  _core?: { _renderService?: { dimensions?: { css?: { cell?: Area } } } };
}

let learned: { cell: Area; extra: Area } | undefined;

export function learnGeometry(terminal: Terminal, tile: Area) {
  const cell = (terminal as unknown as Internals)._core?._renderService
    ?.dimensions?.css?.cell;
  const parent = terminal.element?.parentElement;
  if (!cell || !parent || cell.width === 0 || cell.height === 0) {
    return;
  }

  const scrollbar =
    terminal.options.scrollback === 0
      ? 0
      : (terminal.options.overviewRuler?.width ?? 14);
  learned = {
    cell: { width: cell.width, height: cell.height },
    extra: {
      width: tile.width - parent.clientWidth + scrollbar,
      height: tile.height - parent.clientHeight,
    },
  };
}

export function predictSize(tile: Area | undefined): Size | null {
  if (!learned || !tile) {
    return null;
  }

  const cols = Math.floor(
    (tile.width - learned.extra.width) / learned.cell.width,
  );
  const rows = Math.floor(
    (tile.height - learned.extra.height) / learned.cell.height,
  );

  return cols >= 2 && rows >= 1 ? { cols, rows } : null;
}
