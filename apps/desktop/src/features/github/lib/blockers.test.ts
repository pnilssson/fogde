import { describe, expect, it } from "vitest";

import { blockers, merging } from "@/features/github/lib/blockers";
import type { Check, PullDetail } from "@/generated/protocol";

const check = (name: string, state: Check["state"], required: boolean) => ({
  name,
  state,
  url: null,
  run: null,
  job: null,
  required,
});

const pull = (changes: Partial<PullDetail>): PullDetail => ({
  draft: false,
  head: "feature/login",
  owner: "acme",
  sha: "3f2a9b1c0d4e5f60718293a4b5c6d7e8f9012345",
  base: "main",
  cross: false,
  review: null,
  merge: "blocked",
  methods: ["squash"],
  automerge: true,
  auto: null,
  reviewers: [],
  checks: [],
  threads: [],
  files: [],
  added: 1,
  removed: 1,
  changed: 1,
  ...changes,
});

describe("blockers", () => {
  it("names what stands between a pull request and merging", () => {
    expect(
      blockers(
        pull({
          review: "required",
          checks: [
            check("test", "failure", true),
            check("lint", "failure", false),
            check("build", "pending", true),
          ],
        }),
      ),
    ).toEqual([
      "Fix the failing required check",
      "Wait for a required check to finish",
      "Get an approving review",
    ]);
  });

  it("falls back to the branch rules when nothing else explains a block", () => {
    expect(blockers(pull({}))).toEqual(["Meet the branch rules on main"]);
  });

  it("lists nothing once GitHub says it can merge", () => {
    expect(blockers(pull({ merge: "clean", review: "changes" }))).toEqual([]);
  });
});

describe("merging", () => {
  it("merges now, or when ready where the repo allows auto-merge", () => {
    expect(merging(pull({ merge: "clean" }))).toBe("now");
    expect(merging(pull({ merge: "unstable" }))).toBe("now");
    expect(merging(pull({ merge: "blocked" }))).toBe("later");
    expect(merging(pull({ merge: "behind" }))).toBe("later");
  });

  it("offers no merge GitHub would refuse", () => {
    expect(merging(pull({ merge: "blocked", automerge: false }))).toBe(null);
    expect(merging(pull({ merge: "behind", automerge: false }))).toBe(null);
    expect(merging(pull({ merge: "conflicts" }))).toBe(null);
    expect(merging(pull({ merge: "clean", methods: [] }))).toBe(null);
  });
});
