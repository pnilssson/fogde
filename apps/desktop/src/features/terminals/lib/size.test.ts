import type { FitAddon } from "@xterm/addon-fit";
import { expect, test } from "vitest";

import { measure } from "@/features/terminals/lib/size";

function fit(cols: number, rows: number) {
  return { proposeDimensions: () => ({ cols, rows }) } as unknown as FitAddon;
}

test("a hidden terminal that measures NaN sends no size", () => {
  expect(measure(fit(Number.NaN, Number.NaN))).toBeUndefined();
});

test("a visible terminal sends its measured size", () => {
  expect(measure(fit(80, 24))).toEqual({ cols: 80, rows: 24 });
});
