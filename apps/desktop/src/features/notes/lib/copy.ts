import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import { report } from "@/lib/report";

const notice = createStore<{ copied: boolean }>()(() => ({ copied: false }));

let timer = 0;

export function usePathCopied() {
  return useStore(notice, ({ copied }) => copied);
}

export function copyPath(path: string) {
  navigator.clipboard.writeText(path).then(
    () => {
      notice.setState({ copied: true });
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        notice.setState({ copied: false });
      }, 1500);
    },
    (error: unknown) => {
      report("Could not copy the path", error);
    },
  );
}
