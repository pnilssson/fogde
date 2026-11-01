// @vitest-environment happy-dom
import { Terminal } from "@xterm/xterm";
import { expect, test } from "vitest";

import { trackKeyboard } from "@/features/terminals/lib/keyboard";

const paint =
  "\x18\x1b[!p\x1b[?2026l\x1b[?2004l\x1b[=0;1u\x1b[?1049l\x1b[H\x1b[2J";

async function written(terminal: Terminal, data: string) {
  await new Promise<void>((resolve) => {
    terminal.write(data, resolve);
  });
}

test("a shell gets a plain Enter for Shift+Enter and Ctrl+Enter", async () => {
  const terminal = new Terminal({ allowProposedApi: true });
  const keyboard = trackKeyboard(terminal);

  await written(terminal, `${paint}PS /repos/app> `);

  expect([keyboard.enter("shift"), keyboard.enter("control")]).toEqual([
    "\r",
    "\r",
  ]);
});

test("an agent that asks for kitty keys gets them until it pops its flags", async () => {
  const terminal = new Terminal({ allowProposedApi: true });
  const keyboard = trackKeyboard(terminal);

  await written(terminal, `${paint}\x1b[>1u`);
  const pushed = [keyboard.enter("shift"), keyboard.enter("control")];
  await written(terminal, "\x1b[<u");
  const popped = keyboard.enter("shift");

  expect(pushed).toEqual(["\x1b[13;2u", "\x1b[13;5u"]);
  expect(popped).toBe("\r");
});

test("a repaint after attaching restores the agent's keyboard flags", async () => {
  const terminal = new Terminal({ allowProposedApi: true });
  const keyboard = trackKeyboard(terminal);

  await written(terminal, `${paint}\x1b[=3;1u`);
  const restored = keyboard.enter("shift");
  await written(terminal, paint);

  expect(restored).toBe("\x1b[13;2u");
  expect(keyboard.enter("shift")).toBe("\r");
});

test("flags a full-screen program leaves behind don't reach the shell", async () => {
  const terminal = new Terminal({ allowProposedApi: true });
  const keyboard = trackKeyboard(terminal);

  await written(terminal, `${paint}\x1b[?1049h\x1b[>1u\x1b[?1049l`);

  expect(keyboard.enter("shift")).toBe("\r");
});

test("the alternate screen keeps its own flags while the shell's stay plain", async () => {
  const terminal = new Terminal({ allowProposedApi: true });
  const keyboard = trackKeyboard(terminal);

  await written(terminal, `${paint}\x1b[?1049h\x1b[>1u`);
  const full = keyboard.enter("shift");
  await written(terminal, "\x1b[?1049l");
  const shell = keyboard.enter("shift");
  await written(terminal, "\x1b[?1049h");

  expect([full, shell, keyboard.enter("shift")]).toEqual([
    "\x1b[13;2u",
    "\r",
    "\x1b[13;2u",
  ]);
});

test("resetting before a new watch forgets every screen's flags", async () => {
  const terminal = new Terminal({ allowProposedApi: true });
  const keyboard = trackKeyboard(terminal);
  await written(terminal, `${paint}\x1b[>1u\x1b[?1049h\x1b[>1u`);

  await written(terminal, "\x1bc");
  const main = keyboard.enter("shift");
  await written(terminal, "\x1b[?1049h");

  expect([main, keyboard.enter("shift")]).toEqual(["\r", "\r"]);
});
