export function share(room: number, natural: number[]) {
  const heights = natural.map(() => 0);
  const order = natural
    .map((_, index) => index)
    .sort((a, b) => (natural[a] ?? 0) - (natural[b] ?? 0));
  let left = Math.max(0, room);
  let count = order.length;

  for (const index of order) {
    const fair = Math.floor(left / count);
    const height = Math.min(natural[index] ?? 0, fair);
    heights[index] = height;
    left -= height;
    count -= 1;
  }

  const largest = order.at(-1);
  if (largest !== undefined) {
    heights[largest] = (heights[largest] ?? 0) + left;
  }

  return heights;
}
