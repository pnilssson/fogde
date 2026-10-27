export function cell(offset: number, extent: number, count: number) {
  return Math.min(
    count - 1,
    Math.max(0, Math.floor((offset / extent) * count)),
  );
}
