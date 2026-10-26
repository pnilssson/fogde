import {
  BatteryMediumIcon,
  GaugeIcon,
  StackIcon,
  TimerIcon,
} from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Sparkline } from "@/features/monitor/components/sparkline";
import { type Ping, useLatency } from "@/features/monitor/hooks/use-latency";
import { type Reading, useSamples } from "@/features/monitor/hooks/use-samples";
import { bytes, millis, percent } from "@/features/monitor/lib/format";
import { peaks, scale } from "@/features/monitor/lib/peaks";
import type { Metric } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { barMachine, barQuery, metrics } from "@/lib/monitor";

const TRACK = 48;
const CPU_RANGE = 5;
const MEMORY_RANGE = 0.1;
const WATTS_RANGE = 2;
const LATENCY_RANGE = 20;

interface Meter {
  metric: Metric;
  icon: ReactNode;
  values: number[];
  domain: [number, number];
  value: string;
  title: string;
  detail: string;
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

  return (
    <ul
      className="flex items-center gap-2"
      aria-label={`Fogde on ${machine.name}`}
    >
      {meters.map(({ metric, icon, values, domain, value, title, detail }) => (
        <li key={metric}>
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="sm"
                  className="text-muted-foreground"
                  aria-label={`${title} on ${machine.name}: ${detail}. Open the monitor`}
                  onClick={() => {
                    onOpen(machine);
                  }}
                />
              }
            >
              {icon}
              <Sparkline
                values={values}
                width={TRACK}
                height={16}
                domain={domain}
              />
              <span className="font-mono type-caption tabular-nums">
                {value}
              </span>
            </TooltipTrigger>
            <TooltipContent
              side="top"
              className="max-w-none flex-col items-start"
            >
              <span>
                {title} on {machine.name}
              </span>
              <span>{detail}</span>
            </TooltipContent>
          </Tooltip>
        </li>
      ))}
    </ul>
  );
}

function meter(metric: Metric, samples: Reading[], latency: Ping[]): Meter[] {
  const latest = samples.at(-1);
  const icon = "size-4 shrink-0";
  const series = (pick: (reading: Reading) => number) =>
    peaks(
      samples.map((reading) => ({ time: reading.time, value: pick(reading) })),
    );

  switch (metric) {
    case "cpu": {
      if (!latest) {
        return [];
      }

      const values = series(({ cpu }) => cpu);

      return [
        {
          metric,
          icon: <GaugeIcon className={icon} />,
          values,
          domain: scale(values, CPU_RANGE),
          value: percent(latest.cpu),
          title: "CPU used",
          detail: `${percent(latest.cpu)}, highest ${percent(Math.max(...values))} in 5 minutes`,
        },
      ];
    }
    case "memory": {
      if (!latest) {
        return [];
      }

      const { memory } = latest;
      const values = series((reading) => reading.memory.used);

      return [
        {
          metric,
          icon: <StackIcon className={icon} />,
          values,
          domain: scale(values, memory.used * MEMORY_RANGE),
          value: bytes(memory.used),
          title: "Memory used",
          detail: `${bytes(memory.used)} of ${bytes(memory.total)}, highest ${bytes(Math.max(...values))} in 5 minutes`,
        },
      ];
    }
    case "battery": {
      const battery = latest?.battery;
      if (!battery) {
        return [];
      }

      const values = series((reading) => reading.battery?.watts ?? 0);

      return [
        {
          metric,
          icon: <BatteryMediumIcon className={icon} />,
          values,
          domain: scale(values, WATTS_RANGE),
          value: `${battery.watts.toFixed(0)} W`,
          title: "Battery drain",
          detail: `${battery.watts.toFixed(1)} W, ${percent(battery.drain)} per hour`,
        },
      ];
    }
    case "latency": {
      const last = latency.at(-1);
      if (!last) {
        return [];
      }

      const values = peaks(
        latency.map((ping) => ({ time: ping.time, value: ping.millis })),
      );

      return [
        {
          metric,
          icon: <TimerIcon className={icon} />,
          values,
          domain: scale(values, LATENCY_RANGE),
          value: millis(last.millis),
          title: "Latency",
          detail: `${millis(last.millis)}, highest ${millis(Math.max(...values))} in 5 minutes`,
        },
      ];
    }
  }
}
