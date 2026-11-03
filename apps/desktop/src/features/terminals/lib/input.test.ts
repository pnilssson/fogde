// @vitest-environment happy-dom
import { Terminal } from "@xterm/xterm";
import { expect, test } from "vitest";

import { routeInput } from "@/features/terminals/lib/input";

function routed() {
  const terminal = new Terminal({ allowProposedApi: true });
  const element = document.createElement("div");
  const calls: [string, unknown][] = [];
  const input = routeInput(terminal, element, "t1", (_, method, ...params) => {
    calls.push([method, params[0]]);
  });

  return { terminal, element, calls, input };
}

async function written(terminal: Terminal, data: string) {
  await new Promise<void>((resolve) => {
    terminal.write(data, resolve);
  });
}

test("typing goes to the program as input and the terminal's own answers as replies", async () => {
  const { terminal, calls } = routed();

  terminal.input("ls\r");
  await written(terminal, "\x1b[c\x1b[6n");

  expect(calls).toEqual([
    ["terminals.input", { id: "t1", data: "ls\r" }],
    ["terminals.reply", { id: "t1", data: "\x1b[?1;2c" }],
    ["terminals.reply", { id: "t1", data: "\x1b[1;1R" }],
  ]);
});

test("focus goes to Herdr, which reports it to programs that asked", async () => {
  const { terminal, element, calls } = routed();
  const inside = document.createElement("textarea");
  element.append(inside);

  await written(terminal, "\x1b[?1004h");
  terminal.input("\x1b[I", false);
  element.dispatchEvent(new FocusEvent("focusin"));
  element.dispatchEvent(new FocusEvent("focusout", { relatedTarget: inside }));
  element.dispatchEvent(new FocusEvent("focusout"));

  expect(calls).toEqual([
    ["terminals.focus", { id: "t1", focused: true }],
    ["terminals.focus", { id: "t1", focused: false }],
  ]);
});
