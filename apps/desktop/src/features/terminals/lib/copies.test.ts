// @vitest-environment happy-dom
import { Terminal } from "@xterm/xterm";
import { expect, test, vi } from "vitest";

import { copyFromPrograms } from "@/features/terminals/lib/copies";

async function copiedBy(output: string) {
  const writeText = vi.fn(() => Promise.resolve());
  vi.stubGlobal("navigator", { clipboard: { writeText } });
  const terminal = new Terminal({ allowProposedApi: true });
  copyFromPrograms(terminal, () => undefined);

  await new Promise<void>((resolve) => {
    terminal.write(output, resolve);
  });
  vi.unstubAllGlobals();

  return writeText.mock.calls.map((call: unknown[]) => call[0]);
}

const encoded = btoa(
  String.fromCharCode(...new TextEncoder().encode("feature/login ✓")),
);

test("an agent copies text to the clipboard", async () => {
  expect(await copiedBy(`\x1b]52;c;${encoded}\x07`)).toEqual([
    "feature/login ✓",
  ]);
  expect(await copiedBy("\x1b]52;;" + encoded + "\x1b\\")).toEqual([
    "feature/login ✓",
  ]);
});

test("a program can never read the clipboard or send garbage to it", async () => {
  expect(await copiedBy("\x1b]52;c;?\x07")).toEqual([]);
  expect(await copiedBy("\x1b]52;c;not base64!\x07")).toEqual([]);
  const invalid = btoa(String.fromCharCode(0xff, 0xfe));
  expect(await copiedBy(`\x1b]52;c;${invalid}\x07`)).toEqual([]);
});
