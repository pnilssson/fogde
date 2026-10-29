import type { Terminal } from "@xterm/xterm";

import type { Selection } from "@/features/terminals/lib/mouse";
import type { Key, Platform } from "@/generated/protocol";
import { frame } from "@/lib/frame";

interface Context {
  selection: Selection;
  event: KeyboardEvent;
  send: (key: Key) => void;
}

type Action = (context: Context) => boolean;
type Chords = Partial<Record<string, Action>>;

const modifiers = ["Control", "Alt", "Shift", "Meta"];

function forward(key: Key): Action {
  return ({ selection, event, send }) => {
    if (event.type === "keydown") {
      selection.clear();
      send(key);
    }

    return false;
  };
}

function paste() {
  return false;
}

function copy({ selection, event }: Context) {
  if (!selection.has()) {
    return true;
  }

  if (event.type === "keydown") {
    event.preventDefault();
    selection.copy();
    selection.clear();
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
  macos: { ...named, "meta+c": copy },
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

export function handleKeys(
  terminal: Terminal,
  selection: Selection,
  send: (key: Key) => void,
) {
  terminal.attachCustomKeyEventHandler((event) => {
    const action = chords[chord(event)];
    if (!action || event.isComposing) {
      return true;
    }

    return action({ selection, event, send });
  });
}
