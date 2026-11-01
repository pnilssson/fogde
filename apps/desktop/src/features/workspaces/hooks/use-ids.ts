import { useMemo } from "react";

export function useIds(items: { id: string }[]) {
  const key = items.map(({ id }) => id).join("\n");

  return useMemo(() => (key === "" ? [] : key.split("\n")), [key]);
}
