import { describe, expect, it } from "vitest";

import { nextStep, syncOf } from "@/features/github/lib/sync";
import type { CheckoutStatus } from "@/generated/protocol";

const checkout = (changes: Partial<CheckoutStatus>): CheckoutStatus => ({
  path: "/repos/app",
  repo: { host: "github.com", owner: "acme", name: "app" },
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

describe("syncOf", () => {
  it("follows the branch's state", () => {
    expect(syncOf(checkout({ branch: null })).kind).toBe("detached");
    expect(syncOf(checkout({ upstream: null })).kind).toBe("publish");
    expect(syncOf(checkout({ ahead: 2, behind: 1 })).kind).toBe("diverged");
    expect(syncOf(checkout({ behind: 3 }))).toEqual({
      kind: "pull",
      behind: 3,
    });
    expect(syncOf(checkout({ ahead: 2 }))).toEqual({ kind: "push", ahead: 2 });
    expect(syncOf(checkout({})).kind).toBe("fetch");
  });
});

describe("nextStep", () => {
  const file = { path: "app.txt", from: null, change: "modified" as const };

  it("commits, then pushes, then opens a pull request", () => {
    expect(nextStep(checkout({ files: [file], ahead: 1 }), false, true)).toBe(
      "commit",
    );
    expect(nextStep(checkout({ ahead: 1 }), false, true)).toBe("sync");
    expect(nextStep(checkout({ upstream: null }), false, true)).toBe("sync");
    expect(nextStep(checkout({}), false, true)).toBe("pull-request");
    expect(nextStep(checkout({}), true, true)).toBe(null);
    expect(nextStep(checkout({}), false, false)).toBe(null);
    expect(nextStep(checkout({ ahead: 1, behind: 1 }), false, true)).toBe(
      "pull-request",
    );
  });
});
