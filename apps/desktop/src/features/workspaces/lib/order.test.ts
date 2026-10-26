import { expect, test } from "vitest";

import { arranged, beforeOf, changes } from "@/features/workspaces/lib/order";

const siblings = ["a", "b", "c", "d"];

function drop(moving: string, target: string, edge: "above" | "below") {
  return arranged(siblings, moving, beforeOf(siblings, moving, target, edge));
}

test("dropping below the row above leaves the order unchanged", () => {
  expect(drop("c", "b", "below")).toEqual(["a", "b", "c", "d"]);
  expect(changes(siblings, "c", beforeOf(siblings, "c", "b", "below"))).toBe(
    false,
  );
});

test("dropping above the row below leaves the order unchanged", () => {
  expect(changes(siblings, "b", beforeOf(siblings, "b", "c", "above"))).toBe(
    false,
  );
});

test("dropping below a row puts the terminal after it", () => {
  expect(drop("a", "c", "below")).toEqual(["b", "c", "a", "d"]);
});

test("dropping above a row puts the terminal before it", () => {
  expect(drop("d", "b", "above")).toEqual(["a", "d", "b", "c"]);
});

test("dropping below the last row puts the terminal last", () => {
  expect(drop("a", "d", "below")).toEqual(["b", "c", "d", "a"]);
});
