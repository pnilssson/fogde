import { describe, expect, it } from "vitest";

import { ago, holds, pullFor, samePath } from "@/features/github/lib/scope";
import type { CheckoutStatus } from "@/generated/protocol";

const repo = { host: "github.com", owner: "acme", name: "app" };

const checkout = (changes: Partial<CheckoutStatus>): CheckoutStatus => ({
  path: "/repos/app",
  repo,
  branch: "feature/login",
  upstream: "origin/feature/login",
  ahead: 0,
  behind: 0,
  files: [],
  truncated: false,
  owner: "acme",
  pull: null,
  fetched: null,
  last: null,
  ...changes,
});

const entry = (owner: string | null) => ({
  repo,
  item: {
    number: 7,
    title: "Add login",
    url: "https://github.com/acme/app/pull/7",
    state: "open" as const,
    author: "dev",
    labels: [],
    comments: 0,
    updated: "2026-06-30T12:00:00Z",
    section: "mine" as const,
    pull: {
      draft: false,
      head: "feature/login",
      owner,
      base: "main",
      review: null,
      checks: null,
      added: 1,
      removed: 1,
    },
  },
});

describe("holds", () => {
  const head = { head: "feature/login", owner: "acme", cross: false };

  it("matches a branch named after its fork only for a fork's pull request", () => {
    const prefixed = checkout({ branch: "dev/feature/login", owner: "dev" });

    expect(holds(prefixed, 7, { ...head, owner: "dev", cross: true })).toBe(
      true,
    );
    expect(holds(checkout({ branch: "acme/feature/login" }), 7, head)).toBe(
      false,
    );
  });
});

describe("pullFor", () => {
  it("finds a fork's pull request checked out from its pull ref", () => {
    const fetched = checkout({ pull: 7, upstream: "origin/refs/pull/7/head" });

    expect(pullFor([entry("dev")], fetched)?.item.number).toBe(7);
  });

  it("leaves a branch of the same name in another fork alone", () => {
    expect(pullFor([entry("dev")], checkout({}))).toBeUndefined();
    expect(pullFor([entry("acme")], checkout({}))?.item.number).toBe(7);
  });
});

describe("samePath", () => {
  it("matches the spellings git and Windows give one folder", () => {
    expect(
      samePath(
        "C:/Users/dev/worktrees/app/login",
        "c:\\Users\\dev\\worktrees\\app\\login\\",
      ),
    ).toBe(true);
    expect(samePath("/repos/app", "/repos/app/")).toBe(true);
    expect(samePath("/repos/App", "/repos/app")).toBe(false);
    expect(samePath("/repos/app", "/repos/app-login")).toBe(false);
  });
});

describe("ago", () => {
  const now = Date.parse("2026-06-30T12:00:00Z");
  const day = 86_400_000;

  it("reads the timestamps git, gh and the host send", () => {
    expect(ago("2026-06-30T13:00:00+02:00", now)).toBe("1h ago");
    expect(ago("2026-06-30T11:30:00Z", now)).toBe("30m ago");
    expect(ago(now - 2 * day, now)).toBe("2d ago");
  });

  it("calls a time ahead of this clock just now", () => {
    expect(ago("2026-06-30T12:05:00Z", now)).toBe("just now");
  });

  it("rounds the last days of a year to a year", () => {
    expect(ago(now - 300 * day, now)).toBe("10mo ago");
    expect(ago(now - 362 * day, now)).toBe("1y ago");
    expect(ago(now - 800 * day, now)).toBe("2y ago");
  });
});
