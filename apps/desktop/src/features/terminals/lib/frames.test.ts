// @vitest-environment happy-dom
import { Terminal } from "@xterm/xterm";
import { expect, test } from "vitest";

import { showFrames } from "@/features/terminals/lib/frames";
import type { TerminalFrame } from "@/generated/protocol";
import type { Client } from "@/lib/client";

const cols = 4;
const rows = 2;

function view() {
  const terminal = new Terminal({ allowProposedApi: true, cols, rows });
  let send: (frame: TerminalFrame) => void = () => undefined;
  const client = {
    listen: (_event: string, listener: (frame: TerminalFrame) => void) => {
      send = listener;
      return () => undefined;
    },
  } as unknown as Client;
  showFrames(client, "w1", terminal);

  return {
    frame: (data: string, full = false) => {
      send({ id: "w1", cols, rows, full, data: encode(data) });
    },
    row: (index: number) =>
      terminal.buffer.active.getLine(index)?.translateToString(true),
  };
}

function encode(text: string) {
  return btoa(String.fromCharCode(...new TextEncoder().encode(text)));
}

test("an emoji Herdr draws two cells wide covers the cell beside it", async () => {
  const { frame, row } = view();
  frame("\x1b[?2026h\x1b[2J\x1b[1;1Hab\x1b[?2026l", true);
  frame("\x1b[?2026h\x1b[1;1H⚠️\x1b[?2026l");

  await expect.poll(() => row(0)).toBe("⚠️");
});

test("a character too wide for the last column stays on its row", async () => {
  const { frame, row } = view();
  frame("\x1b[?2026h\x1b[2J\x1b[1;1Habcd\x1b[2;1Hwxyz\x1b[?2026l", true);
  frame("\x1b[?2026h\x1b[1;4H界\x1b[1;1Hq\x1b[?2026l");

  await expect.poll(() => row(0)).toBe("qbcd");
  expect(row(1)).toBe("wxyz");
});
