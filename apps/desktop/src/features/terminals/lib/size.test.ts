import type { FitAddon } from "@xterm/addon-fit";
import { expect, test } from "vitest";

import { measure, sizer } from "@/features/terminals/lib/size";
import type { Size } from "@/generated/protocol";

function fit(cols: number, rows: number) {
  return { proposeDimensions: () => ({ cols, rows }) } as unknown as FitAddon;
}

function box(width: number, height: number) {
  return { clientWidth: width, clientHeight: height } as HTMLElement;
}

test("a hidden terminal that measures NaN sends no size", () => {
  expect(measure(fit(Number.NaN, Number.NaN), box(800, 600))).toBeUndefined();
});

test("a tile squeezed to nothing behind a zoomed tile has no size", () => {
  expect(measure(fit(2, 1), box(0, 600))).toBeUndefined();
  expect(measure(fit(80, 1), box(800, 0))).toBeUndefined();
});

test("a visible terminal sends its measured size", () => {
  expect(measure(fit(80, 24), box(800, 600))).toEqual({ cols: 80, rows: 24 });
});

test("a resize that fails is sent again at the same size", async () => {
  const sent: Size[] = [];
  let failing = true;
  const size = sizer((next) => {
    sent.push(next);

    return failing
      ? Promise.reject(new Error("lost the connection"))
      : Promise.resolve();
  });

  size.started({ cols: 80, rows: 24 });
  await expect(size.resize({ cols: 100, rows: 30 })).rejects.toThrow();
  failing = false;
  await size.resize({ cols: 100, rows: 30 });

  expect(sent).toEqual([
    { cols: 100, rows: 30 },
    { cols: 100, rows: 30 },
  ]);
  expect(size.resize({ cols: 100, rows: 30 })).toBeUndefined();
});
