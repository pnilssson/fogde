import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { useQueryClient } from "@tanstack/react-query";
import type { DockviewPanelApi } from "dockview-react";
import { useEffect, useRef, useState } from "react";

import { toast } from "@/components/ui/toast";
import { workspacesQuery } from "@/features/terminals/api/workspaces";
import {
  type Status,
  TerminalStatus,
} from "@/features/terminals/components/terminal-status";
import { appearance } from "@/features/terminals/lib/appearance";
import { measure } from "@/features/terminals/lib/size";
import { watchTerminal } from "@/features/terminals/lib/watch";
import type { Size } from "@/generated/protocol";
import { ConnectionError } from "@/lib/client";
import type { Machine } from "@/lib/machines";

interface Props {
  id: string;
  machine: Machine;
  panel: DockviewPanelApi;
}

export function TerminalView({ id, machine, panel }: Props) {
  const container = useRef<HTMLDivElement>(null);
  const retry = useRef<() => void>(() => undefined);
  const queries = useQueryClient();
  const [status, setStatus] = useState<Status>("opening");

  useEffect(() => {
    const element = container.current;
    if (!element) {
      return;
    }

    const client = machine.client;
    const options = appearance(element);
    const terminal = new Terminal({
      ...options,
      allowTransparency: true,
      disableStdin: true,
      screenReaderMode: true,
      scrollback: 0,
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);

    let open = true;
    let watching = false;
    let starting = false;
    let stopWatch: (release?: boolean) => void = () => undefined;
    let sent = measure(fit);
    let rendering = Promise.resolve();

    function refresh() {
      void queries.invalidateQueries(workspacesQuery(machine));
    }

    function resize() {
      const size = measure(fit);
      if (!watching || !size || same(size, sent)) {
        return;
      }

      sent = size;
      client.call("terminals.resize", { id, size }).catch((error: unknown) => {
        if (open && watching) {
          report("Could not resize the terminal", error);
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
          terminal.focus();
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

    const stopFrames = client.listen("terminals.frame", (frame) => {
      if (frame.id !== id) {
        return;
      }

      rendering = rendering.then(
        () =>
          new Promise<void>((resolve) => {
            if (!open) {
              resolve();
              return;
            }

            if (frame.cols !== terminal.cols || frame.rows !== terminal.rows) {
              terminal.resize(frame.cols, frame.rows);
            }
            if (frame.full) {
              terminal.write("\x1bc");
            }
            terminal.write(decode(frame.data), resolve);
          }),
      );
    });
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
    const typing = terminal.onData((data) => {
      if (watching) {
        client.call("terminals.input", { id, data }).catch((error: unknown) => {
          if (open && watching) {
            report("Could not send your input", error);
          }
        });
      }
    });
    const focusing = panel.onWillFocus(() => {
      terminal.focus();
    });
    const resizing = new ResizeObserver(resize);
    const theming = new MutationObserver(() => {
      terminal.options = appearance(element);
      resize();
    });
    const shortcuts = (event: KeyboardEvent) => {
      event.stopPropagation();
    };

    resizing.observe(element);
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
      focusing.dispose();
      resizing.disconnect();
      theming.disconnect();
      element.removeEventListener("keydown", shortcuts);
      terminal.dispose();
    };
  }, [id, machine, panel, queries]);

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <TerminalStatus
        machine={machine}
        status={status}
        onRetry={() => {
          retry.current();
        }}
      />
      <div className="terminal-view relative min-h-0 flex-1 p-2 type-code">
        <div ref={container} className="size-full" />
      </div>
    </div>
  );
}

function same(size: Size, other: Size | undefined) {
  return size.cols === other?.cols && size.rows === other.rows;
}

function decode(data: string) {
  return Uint8Array.from(atob(data), (char) => char.charCodeAt(0));
}

function report(title: string, error: unknown) {
  toast.add({
    type: "error",
    title,
    description: error instanceof Error ? error.message : String(error),
  });
}
