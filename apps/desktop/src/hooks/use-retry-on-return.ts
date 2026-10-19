import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import type { Machine } from "@/lib/machines";

export function useRetryOnReturn(machines: Machine[]) {
  const current = useRef(machines);

  useEffect(() => {
    current.current = machines;
  }, [machines]);

  useEffect(() => {
    const retry = () => {
      for (const machine of current.current) {
        const { state, retryable } = machine.client.getSnapshot();
        if (state === "disconnected" && retryable) {
          void machine.client.reconnect().catch(() => undefined);
        }
      }
    };

    window.addEventListener("online", retry);
    const focus = getCurrentWindow().onFocusChanged(({ payload }) => {
      if (payload) {
        retry();
      }
    });

    return () => {
      window.removeEventListener("online", retry);
      void focus.then((stop) => {
        stop();
      });
    };
  }, []);
}
