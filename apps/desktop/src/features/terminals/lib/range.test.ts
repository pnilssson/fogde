import { describe, expect, it } from "vitest";

import { fromBuffer, visible } from "@/features/terminals/lib/range";

const view = { top: 100, rows: 10, cols: 80 };

describe("visible", () => {
  it("selects a range on screen from its first to its last cell", () => {
    const range = {
      anchor: { row: 104, column: 70 },
      cursor: { row: 102, column: 5 },
    };

    expect(visible(range, view)).toEqual({
      column: 5,
      row: 2,
      length: 2 * 80 + 70 - 5 + 1,
    });
  });

  it("keeps the part still on screen after the start scrolls away", () => {
    const range = {
      anchor: { row: 90, column: 40 },
      cursor: { row: 101, column: 9 },
    };

    expect(visible(range, view)).toEqual({ column: 0, row: 0, length: 90 });
  });

  it("fills to the last row when the end is below the screen", () => {
    const range = {
      anchor: { row: 108, column: 78 },
      cursor: { row: 130, column: 0 },
    };

    expect(visible(range, view)).toEqual({ column: 78, row: 8, length: 82 });
  });

  it("shows nothing for a range scrolled entirely off screen", () => {
    const above = {
      anchor: { row: 3, column: 0 },
      cursor: { row: 99, column: 79 },
    };
    const below = {
      anchor: { row: 110, column: 0 },
      cursor: { row: 120, column: 0 },
    };

    expect(visible(above, view)).toBeUndefined();
    expect(visible(below, view)).toBeUndefined();
  });
});

describe("fromBuffer", () => {
  it("reads xterm's exclusive end as the last selected cell", () => {
    expect(
      fromBuffer({ start: { x: 4, y: 2 }, end: { x: 9, y: 2 } }, view),
    ).toEqual({
      anchor: { row: 102, column: 4 },
      cursor: { row: 102, column: 8 },
    });
  });

  it("reads a line selection that ends past the last column", () => {
    expect(
      fromBuffer({ start: { x: 0, y: 3 }, end: { x: 0, y: 4 } }, view),
    ).toEqual({
      anchor: { row: 103, column: 0 },
      cursor: { row: 103, column: 79 },
    });
  });
});
