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

  expect(merge(base, mine, theirs)).toBe(
    "# Plan for login\n\n- login\n- signup\n- reset password\n",
  );
  expect(merge(base, theirs, mine)).toBe(
    "# Plan for login\n\n- login\n- signup\n- reset password\n",
  );
});

test("takes the other side when only one side changed", () => {
  const theirs = `${base}- reset password\n`;

  expect(merge(base, base, theirs)).toBe(theirs);
  expect(merge(base, theirs, base)).toBe(theirs);
  expect(merge(base, theirs, theirs)).toBe(theirs);
});

test("gives up when both sides change the same text", () => {
  const mine = "# Plan\n\n- login with email\n- signup\n";
  const theirs = "# Plan\n\n- login with SSO\n- signup\n";

  expect(merge(base, mine, theirs)).toBeUndefined();
});

test("keeps Windows line endings untouched", () => {
  const windows = "# Plan\r\n\r\n- login\r\n";
  const mine = "# Plan\r\n\r\n- login\r\n- signup\r\n";
  const theirs = "# Plan for login\r\n\r\n- login\r\n";

  expect(merge(windows, mine, theirs)).toBe(
    "# Plan for login\r\n\r\n- login\r\n- signup\r\n",
  );
});
