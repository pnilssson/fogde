import type { DockviewGroupPanel } from "dockview-react";

import { dockview } from "@/features/workspaces/lib/tiles";

function byPlace(groups: DockviewGroupPanel[]) {
  const placed = groups.map((group) => ({
    group,
    rect: group.element.getBoundingClientRect(),
  }));
  placed.sort(
    (a, b) =>
      Math.round(a.rect.left - b.rect.left) ||
      Math.round(a.rect.top - b.rect.top),
  );

  return placed.map(({ group }) => group);
}

export function focusTile(number: number) {
  const api = dockview();
  const group = api && byPlace(api.groups)[number - 1];
  group?.activePanel?.focus();
}
