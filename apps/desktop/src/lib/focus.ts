import { focusManager } from "@tanstack/react-query";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function followWindowFocus() {
  focusManager.setEventListener((setFocused) => {
    const unlisten = getCurrentWindow().onFocusChanged(({ payload }) => {
      setFocused(payload);
    });

    return () => {
      void unlisten.then((stop) => {
        stop();
      });
    };
  });
}
