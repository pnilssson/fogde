import type { IDisposable, IEvent, Terminal } from "@xterm/xterm";

import type { Args, Method } from "@/lib/client";

const focusReports = new Set(["\x1b[I", "\x1b[O"]);

interface Internals {
  _core?: { coreService?: { onUserInput?: IEvent<void> } };
}

type Request = <M extends Method>(
  title: string,
  method: M,
  ...params: Args<M>
) => void;

export interface Input extends IDisposable {
  send: (data: string) => void;
}

export function routeInput(
  terminal: Terminal,
  element: HTMLElement,
  id: string,
  request: Request,
): Input {
  const send = (data: string) => {
    request("Could not send your input", "terminals.input", { id, data });
  };
  const focus = (focused: boolean) => {
    request("Could not focus the terminal", "terminals.focus", {
      id,
      focused,
    });
  };
  const userInput = (terminal as unknown as Internals)._core?.coreService
    ?.onUserInput;
  let typing = false;
  const marking = userInput?.(() => {
    typing = true;
  });
  const data = terminal.onData((input) => {
    const typed = typing || !userInput;
    typing = false;
    if (focusReports.has(input)) {
      return;
    }

    if (typed) {
      send(input);
    } else {
      request("Could not answer the terminal", "terminals.reply", {
        id,
        data: input,
      });
    }
  });
  const focusin = () => {
    focus(true);
  };
  const focusout = (event: FocusEvent) => {
    const next = event.relatedTarget;
    if (!(next instanceof Node && element.contains(next))) {
      focus(false);
    }
  };
  element.addEventListener("focusin", focusin);
  element.addEventListener("focusout", focusout);

  return {
    send,
    dispose: () => {
      marking?.dispose();
      data.dispose();
      element.removeEventListener("focusin", focusin);
      element.removeEventListener("focusout", focusout);
    },
  };
}
