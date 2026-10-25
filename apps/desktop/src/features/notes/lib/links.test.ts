import { expect, test } from "vitest";

import { resolveHref, resolveWikilink } from "@/features/notes/lib/links";
import type { Note } from "@/generated/protocol";

function note(path: string): Note {
  const at = path.lastIndexOf("/");

  return {
    name: path.slice(at + 1).replace(/\.md$/, ""),
    path,
    folder: path.slice(0, at),
    modified: 0,
    created: 0,
  };
}

const plan = note("/notes/app/plan.md");
const otherPlan = note("/notes/web/plan.md");
const login = note("/notes/app/specs/Login flow.md");
const backlog = note("/notes/backlog.md");
const notes = [backlog, otherPlan, plan, login];

test("resolves a wikilink by name, preferring the linking note's folder", () => {
  expect(resolveWikilink("login FLOW", notes, backlog)).toBe(login);
  expect(resolveWikilink("plan", notes, note("/notes/web/ideas.md"))).toBe(
    otherPlan,
  );
  expect(resolveWikilink("plan#Next", notes, login)).toBe(plan);
  expect(resolveWikilink("web/plan.md", notes, plan)).toBe(otherPlan);
  expect(resolveWikilink("missing", notes, plan)).toBeUndefined();
});

test("prefers the shortest path when no folder matches", () => {
  const deep = note("/notes/archive/old/backlog.md");

  expect(resolveWikilink("backlog", [deep, backlog], plan)).toBe(backlog);
});

test("resolves relative markdown links from the note's folder", () => {
  expect(resolveHref("specs/Login%20flow.md", notes, plan)).toBe(login);
  expect(resolveHref("../web/plan.md#Next", notes, plan)).toBe(otherPlan);
  expect(resolveHref("Login%20flow.md", notes, backlog)).toBe(login);
  expect(resolveHref("https://example.com", notes, plan)).toBeUndefined();
  expect(resolveHref("%E0%A4%A.md", notes, plan)).toBeUndefined();
});

test("matches Windows paths whatever their case and separators", () => {
  const windows = {
    ...note("C:/Users/dev/notes/app/plan.md"),
    path: "C:\\Users\\dev\\notes\\app\\plan.md",
    folder: "C:\\Users\\dev\\notes\\app",
  };
  const from = {
    ...windows,
    name: "ideas",
    path: `${windows.folder}\\ideas.md`,
  };

  expect(resolveWikilink("APP/Plan", [windows], from)).toBe(windows);
  expect(resolveHref("./Plan.md", [windows], from)).toBe(windows);
});
