import { focusManager } from "@tanstack/react-query";
import { useEffect, useSyncExternalStore } from "react";

import { useTiles } from "@/features/workspaces/lib/layouts";
import type { Selection } from "@/features/workspaces/lib/selection";
import { useConnected } from "@/hooks/use-connected";
import type { Machine } from "@/lib/machines";
import { usePage } from "@/lib/page";

export function useViewing(machines: Machine[]) {
  const active = useTiles((tiles) => tiles.active);
  const page = usePage();
  const focused = useSyncExternalStore(
    (listener) => focusManager.subscribe(listener),
    () => focusManager.isFocused(),
  );
  const viewing = focused && !page ? active : undefined;

  useConnected(machines, (machine) => {
    view(machine, viewing);
  });

  useEffect(() => {
    for (const machine of machines) {
      view(machine, viewing);
    }
  }, [machines, viewing]);
}

function view(machine: Machine, viewing: Selection | undefined) {
  if (machine.client.getSnapshot().state !== "connected") {
    return;
  }

  const id = viewing?.machine === machine.id ? viewing.id : null;
  machine.client.call("terminals.view", id).catch(() => undefined);
}
