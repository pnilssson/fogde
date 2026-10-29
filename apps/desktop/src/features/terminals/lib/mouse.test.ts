// @vitest-environment happy-dom
import { Terminal } from "@xterm/xterm";
import { expect, test, vi } from "vitest";

import { showFrames } from "@/features/terminals/lib/frames";
import { handleMouse } from "@/features/terminals/lib/mouse";
import type { TerminalFrame, Viewport } from "@/generated/protocol";
import type { Client } from "@/lib/client";

const cols = 80;
const rows = 10;
const cell = 10;
const copied = "line 5456\nline 5457\nline 5458";

function pane(texts: (string | Promise<string>)[] = [copied]) {
  const terminal = new Terminal({ cols, rows });
  const element = document.createElement("div");
  document.body.append(element);
  terminal.open(element);
  const screen = element.querySelector(".xterm-screen") ?? element;
  screen.getBoundingClientRect = () =>
    DOMRect.fromRect({ x: 0, y: 0, width: cols * cell, height: rows * cell });

  let send: (frame: TerminalFrame) => void = () => undefined;
  const reads = [...texts];
  const call = vi.fn(() => Promise.resolve(reads.shift() ?? copied));
  const client = {
    listen: (_event: string, listener: (frame: TerminalFrame) => void) => {
      send = listener;
      return () => undefined;
    },
    call,
  } as unknown as Client;
  const request = vi.fn();
  const selection = handleMouse(terminal, element, {
    client,
    id: "w1",
    request,
    copied: () => undefined,
  });
  showFrames(client, "w1", terminal, selection.drawn);

  const at = (row: number, column: number) => ({
    clientX: column * cell + cell / 2,
    clientY: row * cell + cell / 2,
  });

  return {
    terminal,
    selection,
    request,
    call,
    frame: (viewport: Viewport | null) => {
      send({ id: "w1", cols, rows, full: false, data: "", viewport });
    },
    scroll: (offset: number, max: number) => {
      selection.drawn({ offset, max, rows });
    },
    press: (row: number, column: number, init: MouseEventInit = {}) => {
      element.dispatchEvent(
        new MouseEvent("mousedown", {
          bubbles: true,
          button: 0,
          buttons: 1,
          detail: 1,
          ...at(row, column),
          ...init,
        }),
      );
    },
    move: (row: number, column: number, buttons = 1) => {
      document.dispatchEvent(
        new MouseEvent("mousemove", { buttons, ...at(row, column) }),
      );
    },
    release: (row: number, column: number) => {
      document.dispatchEvent(
        new MouseEvent("mouseup", { button: 0, ...at(row, column) }),
      );
    },
    scrolls: () =>
      request.mock.calls.filter(([, method]) => method === "terminals.scroll")
        .length,
  };
}

function later() {
  let finish: (text: string) => void = () => undefined;
  const text = new Promise<string>((resolve) => {
    finish = resolve;
  });

  return { text, finish };
}

function settle(ms = 0) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

test("a click stays a click when output scrolls the pane while pressed", () => {
  const { selection, request, scroll, press, release } = pane();
  scroll(0, 100);

  press(2, 1);
  scroll(0, 101);
  release(2, 1);

  expect(request).toHaveBeenCalledWith(
    "Could not send your click",
    "terminals.click",
    { id: "w1", row: 2, column: 1 },
  );
  expect(selection.has()).toBe(false);
});

test("a drag ends on release after a modified press lost its release", async () => {
  const { terminal, scroll, press, move, release } = pane();
  scroll(0, 100);

  press(5, 5, { ctrlKey: true });
  press(2, 1);
  move(4, 5);
  release(4, 5);
  await settle();
  move(8, 9);

  expect(terminal.getSelectionPosition()?.end).toEqual({ x: 6, y: 4 });
});

test("dragging past the edge stops scrolling once the button is up", async () => {
  const { scroll, press, move, scrolls } = pane();
  scroll(0, 100);

  press(2, 1);
  move(-1, 1);
  move(-1, 1, 0);
  const sent = scrolls();
  await settle(150);

  expect(scrolls()).toBe(sent);
});

test("dragging past the top stops asking to scroll at the oldest line", async () => {
  const { scroll, press, move, scrolls } = pane();
  scroll(0, 0);

  press(2, 1);
  move(-1, 1);
  await settle(150);

  expect(scrolls()).toBe(0);
});

test("a held selection moves with its text when the frame that scrolled it is drawn", async () => {
  const { terminal, frame, press, move, release } = pane();
  frame({ offset: 0, max: 100, rows });
  await settle(20);
  press(4, 0);
  move(5, 9);
  release(5, 9);
  await settle();

  frame({ offset: 0, max: 102, rows });

  await expect
    .poll(() => terminal.getSelectionPosition())
    .toEqual({
      start: { x: 0, y: 2 },
      end: { x: 10, y: 3 },
    });
});

test("a held selection is cleared once Herdr trims the lines it covered", async () => {
  const { selection, scroll, press, move, release } = pane();
  scroll(0, 714);
  press(2, 0);
  move(4, 9);
  release(4, 9);
  await settle();

  scroll(0, 366);

  expect(selection.has()).toBe(false);
});

test("a held selection is cleared when trimming hid behind new output", async () => {
  const { selection, scroll, press, move, release } = pane([
    copied,
    "line 5800\nline 5801\nline 5802",
  ]);
  scroll(0, 497);
  press(2, 0);
  move(4, 9);
  release(4, 9);
  await settle();

  scroll(0, 611);
  await settle();

  expect(selection.has()).toBe(false);
});

test("a held selection stays while new output leaves its text in place", async () => {
  const { selection, call, scroll, press, move, release } = pane();
  scroll(0, 100);
  press(2, 0);
  move(4, 9);
  release(4, 9);
  await settle();

  scroll(0, 120);
  await settle();

  expect(selection.has()).toBe(true);
  expect(call).toHaveBeenCalledTimes(2);
});

test("a slow copy of an older selection leaves the newer one on the clipboard", async () => {
  const slow = later();
  const { scroll, press, move, release } = pane([slow.text, "line 5460"]);
  const write = vi
    .spyOn(navigator.clipboard, "writeText")
    .mockResolvedValue(undefined);
  scroll(0, 100);
  press(2, 0);
  move(4, 9);
  release(4, 9);
  press(6, 0);
  move(6, 8);
  release(6, 8);
  await settle();

  slow.finish(copied);
  await settle();

  expect(write).toHaveBeenLastCalledWith("line 5460");
});

test("a re-read that returns after the pane closes leaves the terminal alone", async () => {
  const slow = later();
  const { terminal, selection, scroll, press, move, release } = pane([
    copied,
    slow.text,
  ]);
  scroll(0, 100);
  press(2, 0);
  move(4, 9);
  release(4, 9);
  await settle();
  scroll(0, 120);
  selection.dispose();
  const clearing = vi.spyOn(terminal, "clearSelection");

  slow.finish("line 5800\nline 5801\nline 5802");
  await settle();

  expect(clearing).not.toHaveBeenCalled();
});
