import { BatteryMediumIcon } from "@phosphor-icons/react";
import { useEffect } from "react";

import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import { LiveChart } from "@/features/monitor/components/live-chart";
import { Storage } from "@/features/monitor/components/storage";
import { TerminalLoads } from "@/features/monitor/components/terminal-loads";
import { useLatency } from "@/features/monitor/hooks/use-latency";
import { type Reading, useSamples } from "@/features/monitor/hooks/use-samples";
import { bytes, millis, percent } from "@/features/monitor/lib/format";
import { recent } from "@/features/monitor/lib/peaks";
import type { Workspace } from "@/generated/protocol";
import { ConnectionError } from "@/lib/client";
import type { Machine } from "@/lib/machines";

const LIVE = 60_000;

interface Props {
  machine: Machine;
  workspaces: Workspace[];
}

export function MonitorView({ machine, workspaces }: Props) {
  const reading = useSamples(machine);
  const samples = reading.data ?? [];
  const latency = useLatency(machine).data ?? [];
  const latest = samples.at(-1);
  const failure =
    reading.error && !(reading.error instanceof ConnectionError)
      ? reading.error.message
      : undefined;

  useEffect(() => {
    if (failure === undefined) {
      return;
    }

    toast.add({
      type: "error",
      title: "Could not load the monitor",
      description: failure,
    });
  }, [failure]);

  if (!latest) {
    if (failure !== undefined) {
      return null;
    }

    return (
      <div className="flex h-full items-center justify-center">
        <Spinner />
      </div>
    );
  }

  const series = (pick: (reading: Reading) => number) =>
    recent(samples, LIVE).map((reading) => ({
      time: reading.time,
      value: pick(reading),
    }));
  const { memory } = latest;

  return (
    <ScrollArea className="h-full">
      <div className="@container flex flex-col gap-6 p-4">
        <div className="grid gap-4 @xl:grid-cols-2">
          <LiveChart
            label="CPU used"
            points={series(({ cpu }) => cpu)}
            format={percent}
          />
          <LiveChart
            label="Memory used"
            points={series((reading) => reading.memory.used)}
            format={bytes}
            detail={`${bytes(memory.used)} of ${bytes(memory.total)}`}
          />
          {latency.length > 0 && (
            <LiveChart
              label="Latency"
              points={recent(latency, LIVE).map((ping) => ({
                time: ping.time,
                value: ping.millis,
              }))}
              format={millis}
            />
          )}
        </div>
        {latest.battery && (
          <p className="flex items-center gap-2 type-body">
            <BatteryMediumIcon className="size-4 shrink-0" />
            <span>Battery drain</span>
            <span className="ml-auto font-mono tabular-nums">
              {latest.battery.watts.toFixed(1)} W,{" "}
              {percent(latest.battery.drain)} per hour
            </span>
          </p>
        )}
        <TerminalLoads samples={samples} workspaces={workspaces} />
        <Storage machine={machine} workspaces={workspaces} disk={latest.disk} />
      </div>
    </ScrollArea>
  );
}
