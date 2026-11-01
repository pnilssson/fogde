import { expect, test } from "vitest";

import { diff, merge } from "@/features/notes/lib/merge";

const base = "# Plan\n\n- login\n- signup\n";

test("finds the one changed range between two texts", () => {
  expect(diff(base, base)).toBeUndefined();
  expect(diff(base, "# Plan\n\n- login flow\n- signup\n")).toEqual({
    from: 15,
    to: 15,
    insert: " flow",
  });
  expect(diff("aaa", "aa")).toEqual({ from: 2, to: 3, insert: "" });
});

test("keeps both sides when an agent and the user edit different parts", () => {
  const mine = "# Plan for login\n\n- login\n- signup\n";
  const theirs = `${base}- reset password\n`;
  const both = {
    text: "# Plan for login\n\n- login\n- signup\n- reset password\n",
    kept: false,
  };

  expect(merge(base, mine, theirs)).toEqual(both);
  expect(merge(base, theirs, mine)).toEqual(both);
});

test("takes the other side when only one side changed", () => {
  const theirs = `${base}- reset password\n`;
  const taken = { text: theirs, kept: false };

  expect(merge(base, base, theirs)).toEqual(taken);
  expect(merge(base, theirs, base)).toEqual(taken);
  expect(merge(base, theirs, theirs)).toEqual(taken);
});

test("keeps an agent's edits above and below the user's edit", () => {
  const mine = "# Plan\n\n- login with email\n- signup\n";
  const theirs = "# Plan for app\n\n- login\n- signup\n- reset password\n";

  expect(merge(base, mine, theirs)).toEqual({
    text: "# Plan for app\n\n- login with email\n- signup\n- reset password\n",
    kept: false,
  });
});

test("keeps an agent's line added right after the line the user edits", () => {
  const mine = "# Plan\n\n- login with email\n- signup\n";
  const theirs = "# Plan\n\n- login\n- login with SSO\n- signup\n";

  expect(merge(base, mine, theirs)).toEqual({
    text: "# Plan\n\n- login with email\n- login with SSO\n- signup\n",
    kept: false,
  });
});

test("keeps the user's line where both changed it and merges the rest", () => {
  const mine = "# Plan\n\n- login with email\n- signup\n";
  const theirs = "# Plan\n\n- login with SSO\n- signup\n- reset password\n";

  expect(merge(base, mine, theirs)).toEqual({
    text: "# Plan\n\n- login with email\n- signup\n- reset password\n",
    kept: true,
  });
});

test("keeps the agent's lines and then the user's where both added lines in the same place", () => {
  const mine = `${base}- logout\n`;
  const theirs = `${base}- reset password\n`;

  expect(merge(base, mine, theirs)).toEqual({
    text: `${base}- reset password\n- logout\n`,
    kept: false,
  });
  expect(merge(base, mine, mine)).toEqual({ text: mine, kept: false });
});

test("keeps the user's text when an agent rewrote too much of a note to compare", () => {
  const lines = Array.from(
    { length: 30_000 },
    (_, line) => `- ${String(line)}\n`,
  );
  const old = lines.join("");
  const mine = old.replace("- 15000\n", "- logout\n");
  const theirs = lines.map((line) => `* ${line}`).join("");

  expect(merge(old, mine, theirs)).toEqual({ text: mine, kept: true });
});

test("keeps Windows line endings untouched", () => {
  const windows = "# Plan\r\n\r\n- login\r\n";
  const mine = "# Plan\r\n\r\n- login\r\n- signup\r\n";
  const theirs = "# Plan for login\r\n\r\n- login\r\n";

  expect(merge(windows, mine, theirs)).toEqual({
    text: "# Plan for login\r\n\r\n- login\r\n- signup\r\n",
    kept: false,
  });
});

test("merges a last line without a line break", () => {
  expect(merge("a\nb\nc", "a\nB\nc", "A\nb\nc")).toEqual({
    text: "A\nB\nc",
    kept: false,
  });
  expect(merge("a\nb", "a\nb!", "a\nb?")).toEqual({
    text: "a\nb!",
    kept: true,
  });
});

test("keeps both last lines apart when neither side ended with a line break", () => {
  expect(merge("a\n", "a\nmine", "a\ntheirs")).toEqual({
    text: "a\ntheirs\nmine",
    kept: false,
  });
  expect(merge("a\r\n", "a\r\nmine", "a\r\ntheirs")).toEqual({
    text: "a\r\ntheirs\r\nmine",
    kept: false,
  });
  expect(merge("", "mine", "theirs")).toEqual({
    text: "theirs\nmine",
    kept: false,
  });
});
