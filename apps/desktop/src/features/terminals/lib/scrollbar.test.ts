import { expect, test } from "vitest";

import { dragged, place, share } from "@/features/terminals/lib/scrollbar";

const scroll = { offset: 100, max: 300, rows: 100 };

test("the thumb covers the share of the history that is visible", () => {
  expect(share(scroll)).toBe(0.25);
});

test("the thumb sits at the bottom when the terminal follows the output", () => {
  expect(place(scroll, 0)).toBe(1);
  expect(place(scroll, 300)).toBe(0);
});

test("dragging the thumb up scrolls back in proportion to the track", () => {
  expect(dragged(scroll, 0, -75, 150)).toBe(150);
  expect(dragged(scroll, 150, 75, 150)).toBe(0);
});

test("a drag cannot scroll past either end", () => {
  expect(dragged(scroll, 280, -500, 150)).toBe(300);
  expect(dragged(scroll, 20, 500, 150)).toBe(0);
});

test("a track too short to travel leaves the position alone", () => {
  expect(dragged(scroll, 40, 30, 0)).toBe(40);
});
