import { project, type Range } from "@/features/monitor/lib/chart";

const BAR_PAD = 2;
const BAR_GAP = 2;

interface Props {
  values: number[];
  width: number;
  height: number;
  domain?: Range;
}

export function Bars({ values, width, height, domain }: Props) {
  const range = domain ?? [0, Math.max(0, ...values)];
  const y = project(range, BAR_PAD, height - BAR_PAD);
  const base = y(Math.max(range[0], 0));
  const inner = width - BAR_PAD * 2;
  const bar =
    values.length > 0
      ? Math.max(1, (inner - BAR_GAP * (values.length - 1)) / values.length)
      : 0;

  return (
    <svg
      width={width}
      height={height}
      aria-hidden
      className="shrink-0"
      style={{ width, height }}
    >
      {values.map((value, index) => {
        const top = Math.min(base, y(value));

        return (
          <rect
            key={index}
            x={BAR_PAD + index * (bar + BAR_GAP)}
            y={top}
            width={bar}
            height={Math.abs(base - y(value))}
            rx={1}
            fill="currentColor"
          />
        );
      })}
    </svg>
  );
}
