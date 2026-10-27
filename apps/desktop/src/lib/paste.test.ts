import { expect, test } from "vitest";

import { bracket } from "@/lib/paste";

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
