import { ticks } from "d3-array";

export type Range = [number, number];

const TICK_GAP = 36;
const EDGE = 16;

export function extent(values: number[]): Range {
  if (values.length === 0) {
    return [0, 1];
  }

  const low = Math.min(...values);
  const high = Math.max(...values);

  return low === high ? [low - 1, high + 1] : [low, high];
}

export function padded(values: number[]): Range {
  const [low, high] = extent(values);
  const pad = (high - low) * 0.15;

  return [low - pad, high + pad];
}

export function project([low, high]: Range, top: number, bottom: number) {
  return (value: number) =>
    bottom - ((value - low) / (high - low)) * (bottom - top);
}

export function axis(range: Range, height: number) {
  const count = Math.max(2, Math.floor(height / TICK_GAP));

  return ticks(range[0], range[1], count);
}

export function edgeFade(y: number, height: number) {
  return Math.min(1, Math.max(0, Math.min(y, height - y) / EDGE));
}
