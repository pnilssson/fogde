import type { TerminalScrolled } from "@/generated/protocol";

export type Scroll = Omit<TerminalScrolled, "id">;

export function share(scroll: Scroll) {
  return scroll.rows / (scroll.rows + scroll.max);
}

export function place(scroll: Scroll, offset: number) {
  return 1 - offset / scroll.max;
}

export function dragged(
  scroll: Scroll,
  start: number,
  delta: number,
  travel: number,
) {
  if (travel <= 0) {
    return start;
  }

  const offset = Math.round(start - (delta / travel) * scroll.max);

  return Math.min(scroll.max, Math.max(0, offset));
}
