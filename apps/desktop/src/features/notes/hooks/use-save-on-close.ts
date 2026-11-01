import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import { toast } from "@/components/ui/toast";
import { saveAll } from "@/features/notes/lib/session";

const wait = 5000;

let warned = false;
let closing = false;

export async function settle() {
  const late = new Promise<boolean>((resolve) => {
    setTimeout(() => {
      resolve(false);
    }, wait);
  });

  return Promise.race([saveAll().catch(() => false), late]);
}

export function useSaveOnClose() {
  useEffect(() => {
    const stopping = getCurrentWindow().onCloseRequested(async (event) => {
      if (closing) {
        event.preventDefault();
        return;
      }

      const forced = warned;
      closing = true;
      const saved = await settle();
      closing = false;
      if (saved) {
        warned = false;
        return;
      }
      if (forced) {
        return;
      }

      warned = true;
      event.preventDefault();
      toast.add({
        type: "error",
        title: "Could not save a note",
        description: "Close again to quit without saving it.",
      });
    });

    return () => {
      void stopping.then((stop) => {
        stop();
      });
    };
  }, []);
}
