import { area, curveMonotoneX, line } from "d3-shape";
import { type RefObject, useEffect, useId, useRef, useState } from "react";

import { useLive } from "@/features/monitor/hooks/use-live";
import { axis, edgeFade, padded, project } from "@/features/monitor/lib/chart";

const TOP = 12;
const BOTTOM = 12;
const AXIS = 80;
const DOT = 4;
const TIP = 160;
const clock = new Intl.DateTimeFormat(undefined, {
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

export interface Point {
  time: number;
  value: number;
}

interface Props {
  points: Point[];
  span: number;
  height: number;
  label: string;
  format: (value: number) => string;
}

export function LivePlot({ points, span, height, label, format }: Props) {
  const box = useRef<HTMLDivElement>(null);
  const width = useWidth(box);
  const id = useId();
  const [cursor, setCursor] = useState<number>();
  const latest = points.at(-1)?.value ?? 0;
  const live = useLive(latest, padded(points.map(({ value }) => value)));

  const plot = Math.max(0, width - AXIS);
  const floor = height - BOTTOM;
  const start = live.now - span;
  const x = (time: number) => ((time - start) / span) * plot;
  const y = project(live.range, TOP, floor);
  const shown = [
    ...points.filter(({ time }) => time < live.now),
    { time: live.now, value: live.value },
  ];
  const curve = line<Point>()
    .x(({ time }) => x(time))
    .y(({ value }) => y(value))
    .curve(curveMonotoneX)(shown);
  const fill = area<Point>()
    .x(({ time }) => x(time))
    .y0(floor)
    .y1(({ value }) => y(value))
    .curve(curveMonotoneX)(shown);
  const dot = y(live.value);
  const hovered =
    cursor === undefined
      ? undefined
      : nearest(points, start + (cursor / plot) * span);

  return (
    <div
      ref={box}
      className="relative w-full text-foreground"
      style={{ height }}
    >
      {plot > 0 && (
        <svg
          width={width}
          height={height}
          aria-hidden
          className="absolute inset-0"
          onPointerMove={(event) => {
            const left =
              event.clientX - event.currentTarget.getBoundingClientRect().left;
            setCursor(left >= 0 && left <= plot ? left : undefined);
          }}
          onPointerLeave={() => {
            setCursor(undefined);
          }}
        >
          <defs>
            <linearGradient id={`${id}-line`} x1="0" x2="0" y1="0" y2="1">
              <stop offset="0%" stopColor="currentColor" stopOpacity={1} />
              <stop offset="100%" stopColor="currentColor" stopOpacity={0.6} />
            </linearGradient>
            <linearGradient id={`${id}-area`} x1="0" x2="0" y1="0" y2="1">
              <stop offset="0%" stopColor="currentColor" stopOpacity={0.1} />
              <stop offset="100%" stopColor="currentColor" stopOpacity={0} />
            </linearGradient>
            <linearGradient id={`${id}-fade`} x1="0" x2="1" y1="0" y2="0">
              <stop offset="0%" stopColor="white" stopOpacity={0} />
              <stop offset="4%" stopColor="white" stopOpacity={1} />
            </linearGradient>
            <mask id={`${id}-mask`}>
              <rect fill={`url(#${id}-fade)`} width={plot} height={height} />
            </mask>
          </defs>
          <g mask={`url(#${id}-mask)`}>
            {fill && <path d={fill} fill={`url(#${id}-area)`} />}
            {curve && (
              <path
                d={curve}
                fill="none"
                stroke={`url(#${id}-line)`}
                strokeWidth={2}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            )}
          </g>
          <line
            x1={0}
            x2={plot}
            y1={dot}
            y2={dot}
            stroke="currentColor"
            strokeOpacity={0.25}
            strokeDasharray="4,4"
          />
          <g opacity={hovered ? 0.25 : 1}>
            <circle
              cx={plot}
              cy={dot}
              r={DOT + 2}
              fill="currentColor"
              opacity={0.1}
            />
            <circle
              cx={plot}
              cy={dot}
              r={DOT}
              fill="currentColor"
              strokeWidth={2}
              className="stroke-surface-2"
            />
          </g>
          {axis(live.range, floor - TOP).map((tick) => {
            const at = y(tick);

            return (
              <text
                key={tick}
                x={plot + 8}
                y={at}
                dominantBaseline="middle"
                opacity={edgeFade(at - TOP, floor - TOP)}
                className="fill-muted-foreground font-mono type-caption"
              >
                {format(tick)}
              </text>
            );
          })}
          {hovered && (
            <g>
              <line
                x1={x(hovered.time)}
                x2={x(hovered.time)}
                y1={TOP}
                y2={floor}
                stroke="currentColor"
                strokeOpacity={0.3}
                strokeDasharray="4,4"
              />
              <circle
                cx={x(hovered.time)}
                cy={y(hovered.value)}
                r={DOT}
                fill="currentColor"
                strokeWidth={2}
                className="stroke-surface-2"
              />
            </g>
          )}
        </svg>
      )}
      {hovered && cursor !== undefined && (
        <div
          className="pointer-events-none absolute top-0 flex flex-col gap-1 rounded-lg bg-popover px-3 py-2 text-popover-foreground shadow-surface-3"
          style={{
            left: cursor + TIP + 12 > plot ? cursor - TIP - 12 : cursor + 12,
            width: TIP,
          }}
        >
          <span className="type-caption text-muted-foreground">
            {clock.format(hovered.time)}
          </span>
          <span className="flex items-center gap-4 type-caption">
            <span className="text-muted-foreground">{label}</span>
            <span className="ml-auto font-mono tabular-nums">
              {format(hovered.value)}
            </span>
          </span>
        </div>
      )}
    </div>
  );
}

function nearest(points: Point[], time: number) {
  let best: Point | undefined;
  for (const point of points) {
    if (!best || Math.abs(point.time - time) < Math.abs(best.time - time)) {
      best = point;
    }
  }

  return best;
}

function useWidth(ref: RefObject<HTMLDivElement | null>) {
  const [width, setWidth] = useState(0);

  useEffect(() => {
    const element = ref.current;
    if (!element) {
      return;
    }

    const observer = new ResizeObserver(([entry]) => {
      setWidth(entry?.contentRect.width ?? 0);
    });
    observer.observe(element);

    return () => {
      observer.disconnect();
    };
  }, [ref]);

  return width;
}
