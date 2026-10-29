import { FitAddon } from "@xterm/addon-fit";
import { UnicodeGraphemesAddon } from "@xterm/addon-unicode-graphemes";
import { Terminal } from "@xterm/xterm";
import { useQuery } from "@tanstack/react-query";
import type { DockviewPanelApi } from "dockview-react";
import { useEffect, useEffectEvent, useRef, useState } from "react";

import { CopiedNotice } from "@/components/copied-notice";
import { TerminalScrollbar } from "@/features/terminals/components/terminal-scrollbar";
import {
  type Status,
  TerminalStatus,
} from "@/features/terminals/components/terminal-status";
import { useFlash } from "@/features/terminals/hooks/use-flash";
import { useScrolled } from "@/features/terminals/hooks/use-scrolled";
import { appearance } from "@/features/terminals/lib/appearance";
import { handleClipboard } from "@/features/terminals/lib/clipboard";
import { showFrames } from "@/features/terminals/lib/frames";
import { handleKeys } from "@/features/terminals/lib/keys";
import { handleMouse } from "@/features/terminals/lib/mouse";
import { openLinks } from "@/features/terminals/lib/links";
import { report } from "@/lib/report";
import { measure, observeSize, same } from "@/features/terminals/lib/size";
import { watchTerminal } from "@/features/terminals/lib/watch";
import { accessibilityQuery } from "@/lib/accessibility";
import { type Args, ConnectionError, type Method } from "@/lib/client";
import type { Machine } from "@/lib/machines";

interface Props {
  id: string;
  machine: Machine;
  panel: DockviewPanelApi;
  onChange: () => void;
}

export function TerminalView({ id, machine, panel, onChange }: Props) {
  const container = useRef<HTMLDivElement>(null);
  const retry = useRef<() => void>(() => undefined);
  const seek = useRef<(offset: number) => void>(() => undefined);
  const [status, setStatus] = useState<Status>("opening");
  const scroll = useScrolled(machine.client, id);
  const [copied, copy] = useFlash(1500);
  const view = useRef<Terminal>(undefined);
  const reader = useQuery(accessibilityQuery).data?.screenReader ?? false;
  const reading = useEffectEvent(() => reader);
  const refresh = useEffectEvent(onChange);

  useEffect(() => {
    const element = container.current;
    if (!element) {
      return;
    }

    const client = machine.client;
    const options = appearance(element);
    const terminal = new Terminal({
      ...options,
      allowProposedApi: true,
      allowTransparency: true,
      disableStdin: true,
      screenReaderMode: reading(),
      scrollback: 0,
    });
    view.current = terminal;
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.loadAddon(new UnicodeGraphemesAddon());
    openLinks(terminal);

    let open = true;
    let watching = false;
    let starting = false;
    let stopWatch: (release?: boolean) => void = () => undefined;
    let sent = measure(fit);

    function resize() {
      const size = measure(fit);
      if (!watching || !size || same(size, sent)) {
        return;
      }

      sent = size;
      attempt(
        client.call("terminals.resize", { id, size }),
        "Could not resize the terminal",
      );
    }

    function attempt(request: Promise<unknown>, title: string) {
      request.catch((error: unknown) => {
        if (open && watching) {
          report(title, error);
        }
      });
    }

    function show() {
      const size = measure(fit);
      if (
        !open ||
        starting ||
        watching ||
        !size ||
        client.getSnapshot().state === "disconnected"
      ) {
        return;
      }

      starting = true;
      sent = size;
      stopWatch = watchTerminal(
        client,
        id,
        size,
        () => {
          starting = false;
          watching = true;
          terminal.options.disableStdin = false;
          setStatus("ready");
          resize();
          if (panel.isActive && panel.group.api.isActive) {
            terminal.focus();
          }
          refresh();
        },
        (error) => {
          starting = false;
          if (error instanceof ConnectionError) {
            return;
          }

          setStatus("failed");
          report("Could not show the terminal", error);
          refresh();
        },
      );
    }

    retry.current = () => {
      stopWatch();
      setStatus("opening");
      show();
    };

    const stopEnded = client.listen("terminals.ended", (ended) => {
      if (ended.id !== id) {
        return;
      }

      watching = false;
      terminal.options.disableStdin = true;
      setStatus("ended");
      refresh();
      if (ended.error !== null) {
        report("The terminal stopped", ended.error);
      }
    });
    const stopConnection = client.subscribe(() => {
      const state = client.getSnapshot().state;
      if (state === "connected") {
        show();
        return;
      }

      watching = false;
      terminal.options.disableStdin = true;
      if (state !== "connecting") {
        starting = false;
        stopWatch(false);
      }
    });
    function request<M extends Method>(
      title: string,
      method: M,
      ...params: Args<M>
    ) {
      if (watching) {
        attempt(client.call(method, ...params), title);
      }
    }

    function send(data: string) {
      request("Could not send your input", "terminals.input", { id, data });
    }

    const selection = handleMouse(terminal, element, {
      client,
      id,
      request,
      copied: copy,
    });
    const stopFrames = showFrames(client, id, terminal, selection.drawn);
    const typing = terminal.onData(send);
    handleKeys(terminal, selection, (key) => {
      request("Could not send your input", "terminals.key", { id, key });
    });
    const stopClipboard = handleClipboard(element, {
      text: send,
      image: async (image, data) => {
        if (!watching) {
          throw new Error("The terminal is not connected yet.");
        }

        await client.call("terminals.pasteImage", { id, image, data });
      },
      failed: (error) => {
        report("Could not paste", error);
      },
    });
    seek.current = (offset) => {
      request("Could not scroll the terminal", "terminals.seek", {
        id,
        offset,
      });
    };
    const focusing = panel.onWillFocus(() => {
      terminal.focus();
    });
    const resizing = observeSize(element, resize);
    const theming = new MutationObserver(() => {
      terminal.options = appearance(element);
      resize();
    });
    const shortcuts = (event: KeyboardEvent) => {
      event.stopPropagation();
    };

    theming.observe(document.documentElement, { attributeFilter: ["class"] });
    element.addEventListener("keydown", shortcuts);
    void document.fonts
      .load(`${String(options.fontSize)}px ${options.fontFamily}`)
      .catch(() => [])
      .then(() => {
        if (!open) {
          return;
        }

        terminal.open(element);
        show();
      });

    return () => {
      open = false;
      stopConnection();
      stopWatch();
      stopFrames();
      stopEnded();
      typing.dispose();
      stopClipboard();
      selection.dispose();
      focusing.dispose();
      resizing();
      theming.disconnect();
      element.removeEventListener("keydown", shortcuts);
      terminal.dispose();
      view.current = undefined;
    };
  }, [id, machine, panel, copy]);

  useEffect(() => {
    if (view.current) {
      view.current.options.screenReaderMode = reader;
    }
  }, [reader]);

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <TerminalStatus
        machine={machine}
        status={status}
        onRetry={() => {
          retry.current();
        }}
      />
      <div className="terminal-view relative min-h-0 flex-1 px-2 pb-2 type-code">
        <div
          ref={container}
          className="flex size-full flex-col justify-center"
        />
        <TerminalScrollbar
          scroll={scroll}
          onSeek={(offset) => {
            seek.current(offset);
          }}
        />
        <CopiedNotice copied={copied} label="Copied" />
      </div>
    </div>
  );
}
