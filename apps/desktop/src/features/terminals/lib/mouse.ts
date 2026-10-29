import type { Terminal } from "@xterm/xterm";

import { locate } from "@/features/terminals/lib/cell";
import { holdText } from "@/features/terminals/lib/held";
import {
  fromBuffer,
  type Range,
  visible,
} from "@/features/terminals/lib/range";
import { type Scroll, scrollOnWheel } from "@/features/terminals/lib/wheel";
import type { Viewport } from "@/generated/protocol";
import type { Args, Client, Method } from "@/lib/client";
import { report } from "@/lib/report";

type Request = <M extends Method>(
  title: string,
  method: M,
  ...params: Args<M>
) => void;

interface Options {
  client: Client;
  id: string;
  request: Request;
  copied: () => void;
}

export interface Selection {
  has: () => boolean;
  copy: () => void;
  clear: () => void;
  drawn: (viewport: Viewport | null) => void;
  dispose: () => void;
}

const edgeDelay = 50;

export function handleMouse(
  terminal: Terminal,
  element: HTMLElement,
  { client, id, request, copied }: Options,
): Selection {
  let top = 0;
  let offset = 0;
  let max = 0;
  let range: Range | undefined;
  let dragging = false;
  let moved = false;
  let native = false;
  let plain = false;
  let applying = false;
  let pointer = { row: 0, column: 0 };
  let edge: Scroll["direction"] | undefined;
  let edging = 0;
  let timer = 0;
  let copies = 0;

  const scroll = (params: Scroll) => {
    request("Could not scroll the terminal", "terminals.scroll", {
      id,
      ...params,
    });
  };

  const view = () => ({ top, rows: terminal.rows, cols: terminal.cols });

  const selecting = () => range !== undefined && (moved || !dragging);

  const apply = (change: () => void) => {
    applying = true;
    change();
    applying = false;
  };

  const show = () => {
    const part = range && visible(range, view());
    apply(() => {
      if (part) {
        terminal.select(part.column, part.row, part.length);
      } else {
        terminal.clearSelection();
      }
    });
  };

  const clear = () => {
    range = undefined;
    apply(() => {
      terminal.clearSelection();
    });
  };

  const text = holdText(client, id, () => range, clear);

  const adopt = () => {
    const position = terminal.getSelectionPosition();
    range = position && fromBuffer(position, view());
  };

  const copy = () => {
    if (!range || !selecting()) {
      return;
    }

    copies += 1;
    const ticket = copies;
    text
      .read(range)
      .then(async (value) => {
        if (ticket === copies) {
          await navigator.clipboard.writeText(value);
          copied();
        }
      })
      .catch((error: unknown) => {
        report("Could not copy", error);
      });
  };

  const extend = () => {
    if (!range) {
      return;
    }

    const cursor = { row: top + pointer.row, column: pointer.column };
    range = { anchor: range.anchor, cursor };
    if (
      cursor.row !== range.anchor.row ||
      cursor.column !== range.anchor.column
    ) {
      moved = true;
    }
    if (moved) {
      show();
    }
  };

  const follow = (next: Scroll["direction"] | undefined) => {
    edge = next;
    if (!edge) {
      window.clearInterval(edging);
      edging = 0;
      return;
    }
    if (edging) {
      return;
    }

    edging = window.setInterval(() => {
      const room = edge === "up" ? top > 0 : offset > 0;
      if (edge && room) {
        scroll({ direction: edge, lines: 1, ...pointer });
      }
    }, edgeDelay);
  };

  const drawn = (viewport: Viewport | null) => {
    if (viewport) {
      const previous = max;
      ({ offset, max } = viewport);
      top = max - offset;
      if (max < previous) {
        clear();
      } else if (max > previous) {
        text.check();
      }
    }

    if (dragging && (moved || edge)) {
      extend();
    } else if (selecting()) {
      show();
    }
  };

  const finish = () => {
    dragging = false;
    follow(undefined);
    if (!moved) {
      range = undefined;
      if (plain) {
        request("Could not send your click", "terminals.click", {
          id,
          ...pointer,
        });
      }
      return;
    }

    copy();
  };

  const press = (event: MouseEvent) => {
    if (event.button !== 0) {
      return;
    }

    native = event.detail > 1 || event.ctrlKey || event.metaKey || event.altKey;
    if (native) {
      dragging = false;
      follow(undefined);
      return;
    }

    event.stopPropagation();
    event.preventDefault();
    terminal.focus();
    pointer = locate(event, element, terminal).point;
    const point = { row: top + pointer.row, column: pointer.column };
    const extending = event.shiftKey && range !== undefined;
    range = {
      anchor: extending && range ? range.anchor : point,
      cursor: point,
    };
    dragging = true;
    moved = extending;
    plain = !event.shiftKey;
    if (moved) {
      show();
    } else {
      apply(() => {
        terminal.clearSelection();
      });
    }
  };

  const drag = (event: MouseEvent) => {
    if (!dragging) {
      return;
    }

    if ((event.buttons & 1) === 0) {
      finish();
      return;
    }

    const located = locate(event, element, terminal);
    pointer = located.point;
    extend();
    follow(located.edge);
  };

  const release = (event: MouseEvent) => {
    if (event.button !== 0) {
      return;
    }
    if (native) {
      native = false;
      timer = window.setTimeout(() => {
        adopt();
        copy();
      });
      return;
    }
    if (dragging) {
      finish();
    }
  };

  const changing = terminal.onSelectionChange(() => {
    if (!applying && !dragging && terminal.hasSelection()) {
      adopt();
    }
  });
  const typing = terminal.onKey(clear);
  const resizing = terminal.onResize(clear);

  scrollOnWheel(terminal, element, scroll);
  element.addEventListener("mousedown", press, { capture: true });
  document.addEventListener("mousemove", drag);
  document.addEventListener("mouseup", release);

  return {
    has: selecting,
    copy,
    clear,
    drawn,
    dispose: () => {
      range = undefined;
      window.clearTimeout(timer);
      follow(undefined);
      changing.dispose();
      typing.dispose();
      resizing.dispose();
      element.removeEventListener("mousedown", press, { capture: true });
      document.removeEventListener("mousemove", drag);
      document.removeEventListener("mouseup", release);
    },
  };
}
