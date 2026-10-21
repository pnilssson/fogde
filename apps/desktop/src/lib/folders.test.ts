import { expect, test } from "vitest";

import { inside, relative } from "@/lib/folders";

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

test("gives the path below a folder in the path's own spelling", () => {
  expect(relative("/repos/app/src/login", "/repos/app")).toBe("src/login");
  expect(relative("/repos/app/", "/repos/app")).toBe("");
  expect(relative("C:\\Repos\\App\\Src", "c:/repos/app")).toBe("Src");
  expect(relative("/repos/api", "/repos/app")).toBeUndefined();
});
