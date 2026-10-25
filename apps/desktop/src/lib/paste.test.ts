import { expect, test } from "vitest";

import { bracket, pastePath } from "@/lib/paste";

test("a paste is one bracketed paste with plain newlines", () => {
  expect(bracket("one\r\ntwo\rthree\nfour")).toBe(
    "\x1b[200~one\ntwo\nthree\nfour\x1b[201~",
  );
});

test("pasted text cannot end the paste early", () => {
  expect(bracket("safe\x1b[201~rm -rf ~\r\n\x1b[200~")).toBe(
    "\x1b[200~saferm -rf ~\n\x1b[201~",
  );
});

test("a pasted path is quoted only when a shell would split it", () => {
  expect(pastePath("/notes/plan.md")).toBe("\x1b[200~/notes/plan.md\x1b[201~");
  expect(pastePath("C:\\notes\\Login flow.md")).toBe(
    '\x1b[200~"C:\\notes\\Login flow.md"\x1b[201~',
  );
  expect(pastePath("/notes/Q&A;(draft).md")).toBe(
    '\x1b[200~"/notes/Q&A;(draft).md"\x1b[201~',
  );
});
