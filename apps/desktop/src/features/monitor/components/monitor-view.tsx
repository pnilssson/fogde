import { BatteryMediumIcon } from "@phosphor-icons/react";
import { type ReactNode, useEffect } from "react";

import { Body, Main, Scroll } from "@/components/frame";
import { Labelled } from "@/components/labelled";
import { PageBar } from "@/components/page-bar";
import { TileStatus } from "@/components/tile-status";
import { toast } from "@/components/ui/toast";
import { LiveChart } from "@/features/monitor/components/live-chart";
import { Storage } from "@/features/monitor/components/storage";
import { TerminalLoads } from "@/features/monitor/components/terminal-loads";
import { useLatency } from "@/features/monitor/hooks/use-latency";
import { type Reading, useSamples } from "@/features/monitor/hooks/use-samples";
import { bytes, millis, percent } from "@/features/monitor/lib/format";
import { recent } from "@/features/monitor/lib/recent";
import type { Workspace } from "@/generated/protocol";
import { ConnectionError } from "@/lib/client";
import type { Machine } from "@/lib/machines";

const LIVE = 60_000;

interface Props {
  machine: Machine;
  workspaces: Workspace[];
  picker: ReactNode;
}

export function MonitorView({ machine, workspaces, picker }: Props) {
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
    return (
      <>
        <PageBar>
          {picker}
          <span className="flex-1" />
        </PageBar>
        {failure === undefined && (
          <TileStatus notice="Reading the machine…" busy fill />
        )}
      </>
    );
  }

  const series = (pick: (reading: Reading) => number) =>
    recent(samples, LIVE).map((reading) => ({
      time: reading.time,
      value: pick(reading),
    }));
  const { memory } = latest;

  return (
    <>
      <PageBar>
        {picker}
        {latest.battery && (
          <span className="flex min-w-0 items-center gap-2">
            <BatteryMediumIcon aria-hidden className="size-4 shrink-0" />
            <Labelled
              label="Battery drain"
              value={`${latest.battery.watts.toFixed(1)} W, ${percent(latest.battery.drain)} per hour`}
            />
          </span>
        )}
        <span className="flex-1" />
      </PageBar>
      <Body>
        <Main>
          <Scroll>
            <div className="@container flex flex-col gap-4">
              <div className="grid gap-2 @xl:grid-cols-2">
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
              <TerminalLoads samples={samples} workspaces={workspaces} />
              <Storage
                machine={machine}
                workspaces={workspaces}
                disk={latest.disk}
              />
            </div>
          </Scroll>
        </Main>
      </Body>
    </>
  );
}
