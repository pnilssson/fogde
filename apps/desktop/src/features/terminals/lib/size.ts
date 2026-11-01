import type { FitAddon } from "@xterm/addon-fit";

import type { Size } from "@/generated/protocol";

export function measure(fit: FitAddon, element: HTMLElement): Size | undefined {
  if (element.clientWidth === 0 || element.clientHeight === 0) {
    return undefined;
  }

  const size = fit.proposeDimensions();
  if (!size || !(size.cols >= 1 && size.rows >= 1)) {
    return undefined;
  }

  return { cols: size.cols, rows: size.rows };
}

function same(size: Size, other: Size | undefined) {
  return size.cols === other?.cols && size.rows === other.rows;
}

export function observeSize(
  element: HTMLElement,
  changed: () => void,
  wait = 150,
) {
  let timer = 0;
  const observer = new ResizeObserver(() => {
    window.clearTimeout(timer);
    timer = window.setTimeout(changed, wait);
  });
  observer.observe(element);

  return () => {
    window.clearTimeout(timer);
    observer.disconnect();
  };
}

export function sizer(send: (size: Size) => Promise<unknown>) {
  let sent: Size | undefined;

  return {
    started: (size: Size) => {
      sent = size;
    },
    resize: (size: Size) => {
      if (same(size, sent)) {
        return undefined;
      }

      sent = size;
      return send(size).catch((error: unknown) => {
        if (same(size, sent)) {
          sent = undefined;
        }
        throw error;
      });
    },
  };
}
