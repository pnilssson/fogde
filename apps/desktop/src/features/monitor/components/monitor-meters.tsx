import { useQuery } from "@tanstack/react-query";

import { Button } from "@/components/ui/button";
import { type Ping, useLatency } from "@/features/monitor/hooks/use-latency";
import { type Reading, useSamples } from "@/features/monitor/hooks/use-samples";
import { bytes, millis, percent } from "@/features/monitor/lib/format";
import type { Metric } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { barMachine, barQuery, metrics } from "@/lib/monitor";

interface Meter {
  metric: Metric;
  label: string;
  value: string;
}

interface Props {
  machines: Machine[];
  onOpen: (machine: Machine) => void;
}

export function MonitorMeters({ machines, onOpen }: Props) {
  const { data: bar } = useQuery(barQuery);
  const machine = bar && barMachine(bar, machines);
  if (!bar || !machine || bar.metrics.length === 0) {
    return null;
  }

  return <Meters machine={machine} shown={bar.metrics} onOpen={onOpen} />;
}

interface MetersProps {
  machine: Machine;
  shown: Metric[];
  onOpen: (machine: Machine) => void;
}

function Meters({ machine, shown, onOpen }: MetersProps) {
  const sampled = shown.some((metric) => metric !== "latency");
  const samples = useSamples(machine, sampled).data ?? [];
  const latency = useLatency(machine, shown.includes("latency")).data ?? [];
  const meters = metrics
    .filter(({ metric }) => shown.includes(metric))
    .flatMap(({ metric }) => meter(metric, samples, latency));
  if (meters.length === 0) {
    return null;
  }

  const rows = meters.flatMap((_, index) =>
    index % 2 === 0 ? [meters.slice(index, index + 2)] : [],
  );

  return (
    <div className="flex flex-col">
      {rows.map((row) => (
        <Button
          key={row.map(({ metric }) => metric).join()}
          variant="ghost"
          size="sm"
          className="w-full justify-between gap-4 px-2 text-muted-foreground"
          onClick={() => {
            onOpen(machine);
          }}
        >
          {row.map(({ metric, label, value }) => (
            <span
              key={metric}
              className="flex gap-2 font-mono type-caption tabular-nums"
            >
              {label}
              <span>{value}</span>
            </span>
          ))}
        </Button>
      ))}
    </div>
  );
}

function meter(metric: Metric, samples: Reading[], latency: Ping[]): Meter[] {
  const latest = samples.at(-1);

  switch (metric) {
    case "cpu": {
      return latest
        ? [{ metric, label: "CPU", value: percent(latest.cpu) }]
        : [];
    }
    case "memory": {
      return latest
        ? [{ metric, label: "Memory", value: bytes(latest.memory.used) }]
        : [];
    }
    case "battery": {
      const battery = latest?.battery;

      return battery
        ? [
            {
              metric,
              label: "Battery",
              value: `${battery.watts.toFixed(0)} W`,
            },
          ]
        : [];
    }
    case "latency": {
      const last = latency.at(-1);

      return last
        ? [{ metric, label: "Latency", value: millis(last.millis) }]
        : [];
    }
  }
}
