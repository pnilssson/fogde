import type { Terminal } from "@xterm/xterm";

import type { Key, Platform } from "@/generated/protocol";
import { frame } from "@/lib/frame";

interface Context {
  terminal: Terminal;
  event: KeyboardEvent;
  send: (key: Key) => void;
}

type Action = (context: Context) => boolean;
type Chords = Partial<Record<string, Action>>;

const modifiers = ["Control", "Alt", "Shift", "Meta"];

function forward(key: Key): Action {
  return ({ event, send }) => {
    if (event.type === "keydown") {
      send(key);
    }

    return false;
  };
}

function paste() {
  return false;
}

function copy({ terminal, event }: Context) {
  if (!terminal.hasSelection()) {
    return true;
  }

  if (event.type === "keydown") {
    setTimeout(() => {
      terminal.clearSelection();
    });
  }

  return false;
}

const named: Chords = {
  "shift+enter": forward("shiftEnter"),
  "control+enter": forward("ctrlEnter"),
};

const clipboard: Chords = {
  "control+v": paste,
  "control+shift+v": paste,
  "control+c": copy,
};

const platforms: Record<Platform, Chords> = {
  macos: named,
  windows: { ...named, ...clipboard },
  linux: { ...named, ...clipboard },
};

const chords = platforms[frame.platform];

function chord(event: KeyboardEvent) {
  return [
    ...modifiers.filter((modifier) => event.getModifierState(modifier)),
    event.key,
  ]
    .join("+")
    .toLowerCase();
}

export function handleKeys(terminal: Terminal, send: (key: Key) => void) {
  terminal.attachCustomKeyEventHandler((event) => {
    const action = chords[chord(event)];
    if (!action || event.isComposing) {
      return true;
    }

    return action({ terminal, event, send });
  });
}
