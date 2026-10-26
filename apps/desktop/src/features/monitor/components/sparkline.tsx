import { curveMonotoneX, line } from "d3-shape";

import { extent, project, type Range } from "@/features/monitor/lib/chart";

const LINE_PAD = 4;
const BAR_PAD = 2;
const BAR_GAP = 2;

interface Props {
  values: number[];
  width: number;
  height: number;
  domain?: Range;
}

export function Sparkline({ values, width, height, domain }: Props) {
  const y = project(domain ?? extent(values), LINE_PAD, height - LINE_PAD);
  const step =
    values.length > 1 ? (width - LINE_PAD * 2) / (values.length - 1) : 0;
  const path = line<number>()
    .x((_, index) => LINE_PAD + index * step)
    .y(y)
    .curve(curveMonotoneX)(values);

  return (
    <svg
      width={width}
      height={height}
      aria-hidden
      className="shrink-0"
      style={{ width, height }}
    >
      {path && (
        <path
          d={path}
          fill="none"
          stroke="currentColor"
          strokeWidth={1.5}
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
    </svg>
  );
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
