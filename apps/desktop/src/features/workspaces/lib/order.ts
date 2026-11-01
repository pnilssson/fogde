import type { Edge } from "@/features/workspaces/lib/reorder";
import type { RowAction } from "@/features/workspaces/components/row-menu";

export function arranged<T>(items: T[], moving: T, before: T | null) {
  const rest = items.filter((item) => item !== moving);
  const at = before === null ? -1 : rest.indexOf(before);

  return rest.toSpliced(at === -1 ? rest.length : at, 0, moving);
}

export function beforeOf(
  siblings: string[],
  moving: string,
  target: string,
  edge: Edge,
) {
  const rest = siblings.filter((id) => id !== moving);

  return rest[rest.indexOf(target) + Number(edge === "below")] ?? null;
}

export function changes(
  siblings: string[],
  moving: string,
  before: string | null,
) {
  return arranged(siblings, moving, before).some(
    (id, index) => id !== siblings[index],
  );
}

export function moves(
  id: string,
  siblings: string[],
  onArrange: (id: string, before: string | null) => void,
): RowAction[] {
  const index = siblings.indexOf(id);

  return [
    {
      label: "Move up",
      disabled: index <= 0,
      onSelect: () => {
        onArrange(id, siblings[index - 1] ?? null);
      },
    },
    {
      label: "Move down",
      disabled: index === siblings.length - 1,
      onSelect: () => {
        onArrange(id, siblings[index + 2] ?? null);
      },
    },
  ];
}
