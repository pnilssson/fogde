import type { Terminal } from "@xterm/xterm";

import type { Keyboard } from "@/features/terminals/lib/keyboard";
import type { Selection } from "@/features/terminals/lib/selection";
import type { Platform } from "@/generated/protocol";
import { frame } from "@/lib/frame";

interface Context {
  selection: Selection;
  keyboard: Keyboard;
  event: KeyboardEvent;
  send: (data: string) => void;
}

type Action = (context: Context) => boolean;
type Chords = Partial<Record<string, Action>>;

const modifiers = ["Control", "Alt", "Shift", "Meta"];

function enter(modifier: "shift" | "control"): Action {
  return ({ selection, keyboard, event, send }) => {
    if (event.type === "keydown") {
      selection.clear();
      send(keyboard.enter(modifier));
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
  "shift+enter": enter("shift"),
  "control+enter": enter("control"),
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
  keyboard: Keyboard,
  send: (data: string) => void,
) {
  terminal.attachCustomKeyEventHandler((event) => {
    const action = chords[chord(event)];
    if (!action || event.isComposing) {
      return true;
    }

    return action({ selection, keyboard, event, send });
  });
}
