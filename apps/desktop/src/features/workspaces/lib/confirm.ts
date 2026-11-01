import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { Checkout, Terminal, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export type Asked =
  | { kind: "close"; machine: Machine; terminal: Terminal }
  | {
      kind: "remove";
      machine: Machine;
      workspace: Workspace;
      checkout: Checkout;
    };

const confirm = createStore<{ asked: Asked | undefined; open: boolean }>()(
  () => ({ asked: undefined, open: false }),
);

export function ask(asked: Asked) {
  confirm.setState({ asked, open: true });
}

export function setAsking(open: boolean) {
  confirm.setState({ open });
}

export function useAsked() {
  return useStore(confirm);
}
