import { describe, expect, it } from "vitest";

import {
  hiddenReason,
  isGenerated,
  parsed,
  shownFile,
  versionOf,
} from "@/features/github/lib/diff";
import type { FileDiff } from "@/generated/protocol";

const file = (path: string, patch: string | null): FileDiff => ({
  path,
  from: null,
  change: "modified",
  added: 1,
  removed: 1,
  patch,
  binary: false,
  large: false,
});

describe("parsed", () => {
  it("reads a patch as git writes it", () => {
    const diff = parsed(
      file(
        "app.txt",
        "diff --git a/app.txt b/app.txt\nindex 4cb29ea..f04eb26 100644\n--- a/app.txt\n+++ b/app.txt\n@@ -1,3 +1,3 @@\n one\n-two\n+2\n three\n",
      ),
    );

    expect(diff?.name).toBe("app.txt");
    expect(diff?.hunks).toHaveLength(1);
  });

  it("reads a new file the host writes for an untracked file", () => {
    const diff = parsed(
      file(
        "notes/todo.md",
        "diff --git a/notes/todo.md b/notes/todo.md\nnew file mode 100644\n--- /dev/null\n+++ b/notes/todo.md\n@@ -0,0 +1,2 @@\n+one\n+two\n\\ No newline at end of file\n",
      ),
    );

    expect(diff?.name).toBe("notes/todo.md");
    expect(diff?.type).toBe("new");
  });

  it("reads a rename and a path with spaces", () => {
    const renamed = parsed(
      file(
        "new.txt",
        "diff --git a/old.txt b/new.txt\nsimilarity index 71%\nrename from old.txt\nrename to new.txt\nindex 2f06fc5..0d74b8f 100644\n--- a/old.txt\n+++ b/new.txt\n@@ -2,4 +2,4 @@ keep\n this\n file\n here\n-please\n+please!\n",
      ),
    );
    const spaced = parsed(
      file(
        "spaced name.txt",
        "diff --git a/spaced name.txt b/spaced name.txt\nindex 587be6b..e25f181 100644\n--- a/spaced name.txt\t\n+++ b/spaced name.txt\t\n@@ -1 +1 @@\n-x\n+y\n\\ No newline at end of file\n",
      ),
    );

    expect(renamed?.name).toBe("new.txt");
    expect(renamed?.prevName).toBe("old.txt");
    expect(spaced?.hunks).toHaveLength(1);
  });

  it("has nothing to show without a patch", () => {
    expect(parsed(file("logo.png", null))).toBeUndefined();
    expect(hiddenReason({ ...file("logo.png", null), binary: true })).toBe(
      "binary file",
    );
  });
});

describe("versionOf", () => {
  const before = file("app.txt", "@@ -1 +1 @@\n-one\n+two\n");
  const after = file("app.txt", "@@ -1 +1 @@\n-one\n+three\n");

  it("changes when a refreshed patch or the collapse changes, so the view redraws", () => {
    expect(versionOf(before, false)).toBe(versionOf({ ...before }, false));
    expect(versionOf(after, false)).not.toBe(versionOf(before, false));
    expect(versionOf(before, true)).not.toBe(versionOf(before, false));
  });
});

describe("isGenerated", () => {
  it("collapses lockfiles and build output", () => {
    expect(isGenerated("Cargo.lock")).toBe(true);
    expect(isGenerated("apps/desktop/package-lock.json")).toBe(true);
    expect(isGenerated("apps/desktop/src/generated/protocol.ts")).toBe(true);
    expect(isGenerated("dist/app.js")).toBe(true);
    expect(isGenerated("src/build.rs")).toBe(false);
    expect(isGenerated("src/app.ts")).toBe(false);
  });
});

describe("shownFile", () => {
  const tops = new Map([
    ["src/app.ts", 0],
    ["src/login.ts", 2000],
    ["src/logout.ts", 2100],
  ]);
  const ids = [...tops.keys()];
  const shown = (top: number, remaining: number, height = 800) =>
    shownFile(ids, (id) => tops.get(id), { top, height, remaining });

  it("follows the file at the top while far from the end", () => {
    expect(shown(0, 1400)).toBe("src/app.ts");
    expect(shown(500, 900)).toBe("src/app.ts");
  });

  it("reaches the last short files as the view nears the end", () => {
    expect(shown(1300, 100)).toBe("src/login.ts");
    expect(shown(1400, 0)).toBe("src/logout.ts");
    expect(shown(1399.5, 0.5)).toBe("src/logout.ts");
  });

  it("keeps the first file when the content barely scrolls", () => {
    expect(shown(0, 100, 2150)).toBe("src/app.ts");
  });

  it("stops at files the viewer has not placed yet", () => {
    expect(
      shownFile(ids, (id) => (id === "src/app.ts" ? 0 : undefined), {
        top: 1400,
        height: 800,
        remaining: 0,
      }),
    ).toBe("src/app.ts");
  });
});
