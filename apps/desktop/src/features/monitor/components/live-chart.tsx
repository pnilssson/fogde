import { LivePlot, type Point } from "@/features/monitor/components/live-plot";

const SPAN = 60_000;
const HEIGHT = 120;

interface Props {
  label: string;
  points: Point[];
  format: (value: number) => string;
  detail?: string;
}

export function LiveChart({ label, points, format, detail }: Props) {
  const latest = points.at(-1)?.value ?? 0;

  return (
    <figure className="flex min-w-0 flex-col gap-2 rounded-xl bg-surface-2 p-4">
      <figcaption className="flex items-baseline justify-between gap-2">
        <span className="type-body">{label}</span>
        <span className="font-mono type-caption text-muted-foreground tabular-nums">
          {detail ?? format(latest)}
        </span>
      </figcaption>
      <LivePlot
        points={points}
        span={SPAN}
        height={HEIGHT}
        label={label}
        format={format}
      />
    </figure>
  );
}
