import type { Edge } from "@/features/workspaces/lib/drag";

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
