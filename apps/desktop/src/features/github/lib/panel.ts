import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";

import type { DiffScope } from "@/generated/protocol";

interface Panel {
  open: boolean;
  scope: DiffScope;
}

const panel = createStore<Panel>()(() => ({ open: false, scope: "turn" }));

export function usePanel<T>(select: (state: Panel) => T) {
  return useStore(panel, select);
}

export function togglePanel() {
  panel.setState(({ open }) => ({ open: !open }));
}

export function setPanelScope(scope: DiffScope) {
  panel.setState({ scope });
}
