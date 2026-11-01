import { describe, expect, it } from "vitest";

import { authored, holds, landed, pullFor } from "@/features/github/lib/scope";
import type { CheckoutStatus } from "@/generated/protocol";

const repo = { host: "github.com", owner: "acme", name: "app" };

const checkout = (changes: Partial<CheckoutStatus>): CheckoutStatus => ({
  path: "/repos/app",
  repo,
  branch: "feature/login",
  upstream: "origin/feature/login",
  ahead: 0,
  behind: 0,
  commits: 1,
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
      sha: "3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345",
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

describe("landed", () => {
  const sha = "3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345";
  const merged = {
    ...entry("acme"),
    item: { ...entry("acme").item, state: "merged" as const },
  };
  const last = {
    sha,
    subject: "Add login",
    date: "2026-06-30T12:00:00Z",
    undoable: false,
  };

  it("offers to clean up a checkout still at the merged commit", () => {
    expect(landed([merged], checkout({ last }))?.item.number).toBe(7);
  });

  it("keeps a checkout with work the pull request does not have", () => {
    const later = { ...last, sha: "0000000000000000000000000000000000000001" };
    const changed = {
      path: "notes.txt",
      from: null,
      change: "untracked" as const,
    };

    expect(landed([merged], checkout({ last: later }))).toBeUndefined();
    expect(
      landed([merged], checkout({ last, files: [changed] })),
    ).toBeUndefined();
    expect(landed([entry("acme")], checkout({ last }))).toBeUndefined();
  });

  it("keeps the base branch a fork opened its pull request from", () => {
    const fork = {
      ...merged,
      item: {
        ...merged.item,
        pull: { ...merged.item.pull, head: "main", owner: "dev" },
      },
    };

    expect(
      landed([fork], checkout({ last, branch: "main", owner: "dev" })),
    ).toBeUndefined();
  });
});

describe("authored", () => {
  const accounts = [
    { host: "github.com", login: "Dev" },
    { host: "ghe.example.com", login: "reviewer" },
  ];

  it("knows a pull request opened from the signed-in account", () => {
    expect(authored(accounts, "github.com", "dev")).toBe(true);
    expect(authored(accounts, "github.com", "reviewer")).toBe(false);
    expect(authored(accounts, "ghe.example.com", "dev")).toBe(false);
    expect(authored(accounts, "github.com", null)).toBe(false);
    expect(authored(undefined, "github.com", "dev")).toBe(false);
  });
});
