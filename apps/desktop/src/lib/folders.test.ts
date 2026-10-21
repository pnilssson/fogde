import { expect, test } from "vitest";

import { inside } from "@/lib/folders";

test("finds a folder inside another by its parts", () => {
  expect(inside("/worktrees/app/login/src", "/worktrees/app/login")).toBe(true);
  expect(inside("/worktrees/app/login", "/worktrees/app/login/")).toBe(true);
  expect(inside("/worktrees/app/login-old", "/worktrees/app/login")).toBe(
    false,
  );
  expect(inside("/worktrees/app", "/worktrees/app/login")).toBe(false);
  expect(inside("", "/worktrees/app/login")).toBe(false);
});

test("ignores case and separators in Windows paths", () => {
  expect(inside("c:\\users\\dev\\Login\\src", "C:/Users/dev/login")).toBe(true);
});
