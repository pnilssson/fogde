import { openUrl } from "@tauri-apps/plugin-opener";
import { WebLinksAddon } from "@xterm/addon-web-links";
import type { Terminal } from "@xterm/xterm";

import { toast } from "@/components/ui/toast";

export function openLinks(terminal: Terminal) {
  const open = (event: MouseEvent, uri: string) => {
    if (!(event.ctrlKey || event.metaKey) || !/^https?:\/\//i.test(uri)) {
      return;
    }

    openUrl(uri).catch((error: unknown) => {
      toast.add({
        type: "error",
        title: "Could not open the link",
        description: error instanceof Error ? error.message : String(error),
      });
    });
  };

  terminal.options.linkHandler = { activate: open };
  terminal.loadAddon(new WebLinksAddon(open));
}
