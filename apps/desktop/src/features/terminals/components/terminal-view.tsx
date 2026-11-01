import { FitAddon } from "@xterm/addon-fit";
import { UnicodeGraphemesAddon } from "@xterm/addon-unicode-graphemes";
import { Terminal } from "@xterm/xterm";
import { useQuery } from "@tanstack/react-query";
import type { DockviewPanelApi } from "dockview-react";
import { useEffect, useEffectEvent, useRef, useState } from "react";

import { CopiedNotice } from "@/components/copied-notice";
import { flashCopied, useCopied } from "@/lib/copied";
import {
  type Status,
  TerminalStatus,
} from "@/features/terminals/components/terminal-status";
import { appearance } from "@/features/terminals/lib/appearance";
import { handleClipboard } from "@/features/terminals/lib/clipboard";
import { trackKeyboard } from "@/features/terminals/lib/keyboard";
import { handleKeys } from "@/features/terminals/lib/keys";
import { openLinks } from "@/features/terminals/lib/links";
import { showOutput } from "@/features/terminals/lib/output";
import { handleSelection } from "@/features/terminals/lib/selection";
import { report } from "@/lib/report";
import { measure, observeSize, sizer } from "@/features/terminals/lib/size";
import { watchTerminal } from "@/features/terminals/lib/watch";
import { accessibilityQuery } from "@/lib/accessibility";
import type { Size } from "@/generated/protocol";
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
  const [status, setStatus] = useState<Status>("opening");
  const place = `terminal:${machine.id}:${id}`;
  const copied = useCopied(place);
  const view = useRef<Terminal>(undefined);
  const reader = useQuery(accessibilityQuery).data?.screenReader ?? false;
  const reading = useEffectEvent(() => reader);
  const refresh = useEffectEvent(onChange);

  useEffect(() => {
    const element = container.current;
    if (!element) {
      return;
    }

    const frame: HTMLElement = element;
    const client = machine.client;
    const options = appearance(element);
    const terminal = new Terminal({
      ...options,
      allowProposedApi: true,
      allowTransparency: true,
      disableStdin: true,
      screenReaderMode: reading(),
      scrollback: 10000,
    });
    view.current = terminal;
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.loadAddon(new UnicodeGraphemesAddon());
    openLinks(terminal);

    const keyboard = trackKeyboard(terminal);
    let open = true;
    let watching = false;
    let starting = false;
    let deferred = false;
    let token: number | undefined;
    let stopWatch: (release?: boolean) => void = () => undefined;
    const sizes = sizer((size) =>
      client.call("terminals.resize", { id, size }),
    );

    function fitTo(size: Size) {
      if (terminal.cols !== size.cols || terminal.rows !== size.rows) {
        terminal.resize(size.cols, size.rows);
      }
    }

    function resize() {
      const size = measure(fit, frame);
      if (size) {
        fitTo(size);
      }
      if (size && deferred) {
        show();
        return;
      }
      const sending = watching && size ? sizes.resize(size) : undefined;
      if (sending) {
        attempt(sending, "Could not resize the terminal");
      }
    }

    function attempt(request: Promise<unknown>, title: string) {
      request.catch((error: unknown) => {
        if (open && watching) {
          report(title, error);
        }
      });
    }

    function show() {
      const size = measure(fit, frame);
      deferred = open && !starting && !watching && !size;
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
      token = undefined;
      fitTo(size);
      stopWatch = watchTerminal(
        client,
        id,
        () => {
          const now = measure(fit, frame) ?? size;
          fitTo(now);
          sizes.started(now);

          return now;
        },
        () => {
          starting = false;
          watching = true;
          terminal.options.disableStdin = false;
          setStatus("ready");
          resize();
          if (panel.isActive && panel.group.api.isActive) {
            terminal.focus();
          }
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
        (next) => {
          terminal.write("\x1bc");
          token = next;
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

    const selection = handleSelection(terminal, element, () => {
      flashCopied(place);
    });
    const stopOutput = showOutput(client, id, terminal, () => token);
    const typing = terminal.onData(send);
    handleKeys(terminal, selection, keyboard, send);
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
      stopOutput();
      keyboard.dispose();
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
  }, [id, machine, panel, place]);

  useEffect(() => {
    if (view.current) {
      view.current.options.screenReaderMode = reader;
    }
  }, [reader]);

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div className="absolute inset-x-0 top-0 z-10 bg-surface-3">
        <TerminalStatus
          machine={machine}
          status={status}
          onRetry={() => {
            retry.current();
          }}
        />
      </div>
      <div className="terminal-view relative min-h-0 flex-1 px-2 pb-2 type-code">
        <div
          ref={container}
          className="flex size-full flex-col justify-center"
        />
        <CopiedNotice copied={copied} label="Copied" />
      </div>
    </div>
  );
}
