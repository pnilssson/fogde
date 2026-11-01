import {
  CpuIcon,
  type Icon,
  LightningIcon,
  StackSimpleIcon,
  PulseIcon,
  WifiHighIcon,
} from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";

import { NavItem } from "@/components/nav-item";
import { type Ping, useLatency } from "@/features/monitor/hooks/use-latency";
import { type Reading, useSamples } from "@/features/monitor/hooks/use-samples";
import { bytes, millis, percent } from "@/features/monitor/lib/format";
import type { Metric } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { barMachine, barQuery, metrics } from "@/lib/monitor";

interface Shown {
  metric: Metric;
  label: string;
  icon: Icon;
  value: string;
}

interface Props {
  machines: Machine[];
  active: boolean;
  onOpen: (machine: Machine) => void;
}

export function MonitorItem({ machines, active, onOpen }: Props) {
  const { data: bar } = useQuery(barQuery);
  const machine = bar && barMachine(bar, machines);
  if (!bar || !machine) {
    return null;
  }

  return (
    <NavItem
      icon={<PulseIcon />}
      label="Monitor"
      active={active}
      trailing={<Values machine={machine} shown={bar.metrics} />}
      onClick={() => {
        onOpen(machine);
      }}
    />
  );
}

function Values({ machine, shown }: { machine: Machine; shown: Metric[] }) {
  const sampled = shown.some((metric) => metric !== "latency");
  const samples = useSamples(machine, sampled).data ?? [];
  const latency = useLatency(machine, shown.includes("latency")).data ?? [];
  const values = metrics
    .filter(({ metric }) => shown.includes(metric))
    .flatMap(({ metric, label }) => value(metric, label, samples, latency))
    .slice(0, 2);

  return values.map(({ metric, label, icon: Glyph, value }) => (
    <span
      key={metric}
      className="flex shrink-0 items-center gap-2 font-mono type-caption tabular-nums"
    >
      <Glyph aria-hidden />
      <span className="sr-only">{label}</span>
      {value}
    </span>
  ));
}

function value(
  metric: Metric,
  label: string,
  samples: Reading[],
  latency: Ping[],
): Shown[] {
  const latest = samples.at(-1);

  switch (metric) {
    case "cpu": {
      return latest
        ? [{ metric, label, icon: CpuIcon, value: percent(latest.cpu) }]
        : [];
    }
    case "memory": {
      return latest
        ? [
            {
              metric,
              label,
              icon: StackSimpleIcon,
              value: bytes(latest.memory.used),
            },
          ]
        : [];
    }
    case "battery": {
      const battery = latest?.battery;

      return battery
        ? [
            {
              metric,
              label,
              icon: LightningIcon,
              value: `${battery.watts.toFixed(0)} W`,
            },
          ]
        : [];
    }
    case "latency": {
      const last = latency.at(-1);

      return last
        ? [{ metric, label, icon: WifiHighIcon, value: millis(last.millis) }]
        : [];
    }
  }
}
