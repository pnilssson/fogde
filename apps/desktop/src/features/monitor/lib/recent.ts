export function recent<T extends { time: number }>(points: T[], span: number) {
  const latest = points.at(-1);

  return latest ? points.filter(({ time }) => time > latest.time - span) : [];
}
