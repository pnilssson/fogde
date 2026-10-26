const units = ["B", "KB", "MB", "GB", "TB"];

export function bytes(value: number) {
  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }

  const digits = unit > 0 && size < 10 ? 1 : 0;

  return `${size.toFixed(digits)} ${units[unit] ?? ""}`;
}

export function percent(value: number) {
  return `${value < 10 ? value.toFixed(1) : String(Math.round(value))}%`;
}

export function millis(value: number) {
  return `${String(Math.round(value))} ms`;
}
