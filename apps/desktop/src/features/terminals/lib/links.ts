import { openUrl } from "@tauri-apps/plugin-opener";
import { WebLinksAddon } from "@xterm/addon-web-links";
import type { Terminal } from "@xterm/xterm";

import { report } from "@/lib/report";

export function openLinks(terminal: Terminal) {
  const open = (event: MouseEvent, uri: string) => {
    if (!(event.ctrlKey || event.metaKey) || !/^https?:\/\//i.test(uri)) {
      return;
    }

    openUrl(uri).catch((error: unknown) => {
      report("Could not open the link", error);
    });
  };

  terminal.options.linkHandler = { activate: open };
  terminal.loadAddon(new WebLinksAddon(open));
}
