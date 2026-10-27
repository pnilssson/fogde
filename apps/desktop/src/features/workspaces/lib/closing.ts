import { useEffect } from "react";

import { closePage, pageOpen } from "@/lib/page";

let closer: (() => void) | undefined;

export function useActiveClose(
  active: boolean,
  close: (() => void) | undefined,
) {
  useEffect(() => {
    if (!active || !close) {
      return;
    }

    closer = close;

    return () => {
      if (closer === close) {
        closer = undefined;
      }
    };
  }, [active, close]);
}

export function closeActive() {
  if (pageOpen()) {
    closePage();
    return;
  }

  closer?.();
}
