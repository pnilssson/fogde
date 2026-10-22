import type { Terminal } from "@xterm/xterm";

import type { Key } from "@/generated/protocol";
import { frame } from "@/lib/frame";

export function handleKeys(terminal: Terminal, send: (key: Key) => void) {
  terminal.attachCustomKeyEventHandler((event) => {
    if (shiftEnter(event)) {
      if (event.type === "keydown") {
        send("shiftEnter");
      }
      return false;
    }

    return clipboard(terminal, event);
  });
}

function shiftEnter(event: KeyboardEvent) {
  return (
    event.key === "Enter" &&
    event.shiftKey &&
    !event.ctrlKey &&
    !event.altKey &&
    !event.metaKey &&
    !event.isComposing
  );
}

function clipboard(terminal: Terminal, event: KeyboardEvent) {
  if (
    frame.platform === "macos" ||
    !event.ctrlKey ||
    event.altKey ||
    event.metaKey
  ) {
    return true;
  }

  const key = event.key.toLowerCase();
  if (key === "v") {
    return false;
  }
  if (key === "c" && !event.shiftKey && terminal.hasSelection()) {
    if (event.type === "keydown") {
      setTimeout(() => {
        terminal.clearSelection();
      });
    }
    return false;
  }

  return true;
}
