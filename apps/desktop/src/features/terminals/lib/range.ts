import type { IBufferRange } from "@xterm/xterm";

import type { TextPoint } from "@/generated/protocol";

export interface Range {
  anchor: TextPoint;
  cursor: TextPoint;
}

export interface View {
  top: number;
  rows: number;
  cols: number;
}

export function ordered({ anchor, cursor }: Range): [TextPoint, TextPoint] {
  const forward =
    anchor.row < cursor.row ||
    (anchor.row === cursor.row && anchor.column <= cursor.column);

  return forward ? [anchor, cursor] : [cursor, anchor];
}

export function visible(range: Range, view: View) {
  const [start, end] = ordered(range);
  const bottom = view.top + view.rows - 1;
  if (end.row < view.top || start.row > bottom) {
    return undefined;
  }

  const first = start.row < view.top ? { row: view.top, column: 0 } : start;
  const last = end.row > bottom ? { row: bottom, column: view.cols - 1 } : end;

  return {
    column: first.column,
    row: first.row - view.top,
    length: (last.row - first.row) * view.cols + last.column - first.column + 1,
  };
}

export function fromBuffer({ start, end }: IBufferRange, view: View): Range {
  const last = end.y * view.cols + end.x - 1;

  return {
    anchor: { row: view.top + start.y, column: start.x },
    cursor: {
      row: view.top + Math.floor(last / view.cols),
      column: last % view.cols,
    },
  };
}
