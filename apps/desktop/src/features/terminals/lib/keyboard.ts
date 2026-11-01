import type { IDisposable, Terminal } from "@xterm/xterm";

const disambiguate = 1;
const limit = 16;

export interface Keyboard extends IDisposable {
  enter: (modifier: "shift" | "control") => string;
}

export function trackKeyboard(terminal: Terminal): Keyboard {
  const stacks = { normal: [0], alternate: [0] };
  const screen = () => stacks[terminal.buffer.active.type];
  const current = () => screen().at(-1) ?? 0;
  const reset = () => {
    stacks.normal = [0];
    stacks.alternate = [0];
  };
  const flags = (params: (number | number[])[], index: number) => {
    const value = params[index];

    return typeof value === "number" ? value : 0;
  };

  const handlers = [
    terminal.parser.registerCsiHandler(
      { prefix: ">", final: "u" },
      (params) => {
        const stack = screen();
        stack.push(flags(params, 0));
        if (stack.length > limit + 1) {
          stack.splice(1, 1);
        }

        return true;
      },
    ),
    terminal.parser.registerCsiHandler(
      { prefix: "<", final: "u" },
      (params) => {
        const stack = screen();
        const count = Math.max(1, flags(params, 0));
        stack.splice(Math.max(1, stack.length - count));

        return true;
      },
    ),
    terminal.parser.registerCsiHandler(
      { prefix: "=", final: "u" },
      (params) => {
        const stack = screen();
        const value = flags(params, 0);
        const mode = flags(params, 1) || 1;
        const next =
          mode === 1
            ? value
            : mode === 2
              ? current() | value
              : current() & ~value;
        stack[stack.length - 1] = next;

        return true;
      },
    ),
  ];
  const resetting = terminal.parser.registerEscHandler({ final: "c" }, () => {
    reset();

    return false;
  });

  return {
    enter: (modifier) => {
      if ((current() & disambiguate) === 0) {
        return "\r";
      }

      return modifier === "shift" ? "\x1b[13;2u" : "\x1b[13;5u";
    },
    dispose: () => {
      for (const handler of [...handlers, resetting]) {
        handler.dispose();
      }
    },
  };
}
