const BUCKET = 5000;
const BUCKETS = 60;

interface Point {
  time: number;
  value: number;
}

export function peaks(points: Point[]) {
  const latest = points.at(-1);
  if (!latest) {
    return [];
  }

  const first = Math.floor(latest.time / BUCKET) - BUCKETS;
  const found = new Map<number, number>();
  for (const { time, value } of points) {
    const bucket = Math.floor(time / BUCKET);
    if (bucket <= first) {
      continue;
    }

    found.set(bucket, Math.max(found.get(bucket) ?? value, value));
  }

  const values = [...found.values()];
  const start = values[0] ?? latest.value;

  return [...Array<number>(BUCKETS - values.length).fill(start), ...values];
}

export function scale(values: number[], least: number): [number, number] {
  const low = Math.min(...values);
  const high = Math.max(...values);
  if (high - low >= least) {
    return [low, high];
  }

  const bottom = Math.max(0, (low + high - least) / 2);

  return [bottom, bottom + least];
}

export function recent<T extends { time: number }>(points: T[], span: number) {
  const latest = points.at(-1);

  return latest ? points.filter(({ time }) => time > latest.time - span) : [];
}
