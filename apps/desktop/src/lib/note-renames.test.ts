import { expect, test } from "vitest";

import { moved } from "@/lib/note-renames";

const root = "C:\\notes";

test("follows a note when it or a folder above it is renamed", () => {
  const plan = `${root}\\app\\plan.md`;

  expect(moved(plan, plan, `${root}\\app\\login.md`)).toBe(
    `${root}\\app\\login.md`,
  );
  expect(moved(plan, `${root}\\app`, `${root}\\web`)).toBe(
    `${root}\\web\\plan.md`,
  );
  expect(moved("/notes/app/plan.md", "/notes/app", "/notes/web")).toBe(
    "/notes/web/plan.md",
  );
  expect(
    moved(`${root}\\apps\\plan.md`, `${root}\\app`, `${root}\\web`),
  ).toBeUndefined();
});
