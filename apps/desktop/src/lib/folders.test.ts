import { expect, test } from "vitest";

import { relative, samePath } from "@/lib/folders";

test("finds a folder inside another by its parts", () => {
  expect(relative("/worktrees/app/login/src", "/worktrees/app/login")).toBe(
    "src",
  );
  expect(relative("/worktrees/app/login", "/worktrees/app/login/")).toBe("");
  expect(
    relative("/worktrees/app/login-old", "/worktrees/app/login"),
  ).toBeUndefined();
  expect(relative("/worktrees/app", "/worktrees/app/login")).toBeUndefined();
  expect(relative("", "/worktrees/app/login")).toBeUndefined();
});

test("ignores case and separators in Windows paths", () => {
  expect(relative("c:\\users\\dev\\Login\\src", "C:/Users/dev/login")).toBe(
    "src",
  );
});

test("gives the path below a folder in the path's own spelling", () => {
  expect(relative("/repos/app/src/login", "/repos/app")).toBe("src/login");
  expect(relative("/repos/app/", "/repos/app")).toBe("");
  expect(relative("C:\\Repos\\App\\Src", "c:/repos/app")).toBe("Src");
  expect(relative("/repos/api", "/repos/app")).toBeUndefined();
});

test("matches the spellings git and Windows give one folder", () => {
  expect(
    samePath(
      "C:/Users/dev/worktrees/app/login",
      "c:\\Users\\dev\\worktrees\\app\\login\\",
    ),
  ).toBe(true);
  expect(samePath("\\\\server\\Share\\app", "//server/share/app")).toBe(true);
  expect(samePath("/repos/app", "/repos/app/")).toBe(true);
  expect(samePath("/repos/app/src/..", "/repos/app")).toBe(true);
  expect(samePath("/repos/App", "/repos/app")).toBe(false);
  expect(samePath("/repos/app", "/repos/app-login")).toBe(false);
});
