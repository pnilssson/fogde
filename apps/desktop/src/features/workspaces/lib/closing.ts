import { useEffect } from "react";

import { closePage, currentPage } from "@/lib/page";

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
  const page = currentPage();
  if (page === "monitor") {
    closePage();
    return;
  }

  if (page) {
    return;
  }

  closer?.();
}
