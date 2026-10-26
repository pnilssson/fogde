import { Progress } from "@base-ui/react/progress";
import { useEffect, useState } from "react";

import { AgentIcon } from "@/components/agent-icon";
import { Button } from "@/components/ui/button";
import { ProgressIndicator, ProgressTrack } from "@/components/ui/progress";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useUsage } from "@/features/usage/hooks/use-usage";
import type { Agent, Span, Window } from "@/generated/protocol";
import { agents } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

const TICK = 30_000;
const SESSION = 5 * 60 * 60_000;

const spans: Record<Span, string> = {
  session: "5-hour",
  week: "Weekly",
  month: "Monthly",
};

interface Props {
  machines: Machine[];
}

export function UsageMeters({ machines }: Props) {
  const usage = useUsage(machines);
  const now = useNow();
  const current = usage.flatMap(({ agent, windows }) => {
    const open = windows.filter(({ resets }) => resets * 1000 > now);
    const shown = open.find(({ span }) => span === "session") ?? open.at(0);

    return shown ? [{ agent, windows: open, shown }] : [];
  });

  if (current.length === 0) {
    return null;
  }

  return (
    <ul className="flex items-center gap-2" aria-label="Usage left">
      {current.map(({ agent, windows, shown }) => (
        <li key={agent}>
          <Meter agent={agent} windows={windows} shown={shown} now={now} />
        </li>
      ))}
    </ul>
  );
}

interface MeterProps {
  agent: Agent;
  windows: Window[];
  shown: Window;
  now: number;
}

function Meter({ agent, windows, shown, now }: MeterProps) {
  const lines = windows.map((window) => describe(window, now));
  const left = Math.floor(shown.left);

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground"
            aria-label={`${agents[agent].short}: ${lines.join(", ")}`}
          />
        }
      >
        <AgentIcon agent={agent} />
        <Progress.Root value={left} aria-hidden>
          <ProgressTrack className="h-1 w-12">
            <ProgressIndicator className="rounded-full bg-muted-foreground" />
          </ProgressTrack>
        </Progress.Root>
        <span className="flex gap-2 font-mono type-caption tabular-nums">
          <span>{left}%</span>
          <span>{remaining(shown, now)}</span>
        </span>
      </TooltipTrigger>
      <TooltipContent side="top">
        {lines.map((line) => (
          <span key={line} className="block">
            {line}
          </span>
        ))}
      </TooltipContent>
    </Tooltip>
  );
}

function useNow() {
  const [now, setNow] = useState(Date.now);

  useEffect(() => {
    const timer = setInterval(() => {
      setNow(Date.now());
    }, TICK);

    return () => {
      clearInterval(timer);
    };
  }, []);

  return now;
}

function describe(window: Window, now: number) {
  const left = String(Math.floor(window.left));

  return `${spans[window.span]} limit ${left}% left, resets in ${remaining(window, now)} (${resetTime(window, now)})`;
}

function resetsAt(window: Window, now: number) {
  const at = window.resets * 1000;

  return window.span === "session" ? Math.min(at, now + SESSION) : at;
}

function remaining(window: Window, now: number) {
  return until(resetsAt(window, now) - now);
}

function until(milliseconds: number) {
  const minutes = Math.max(1, Math.ceil(milliseconds / 60_000));
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor((minutes % 1440) / 60);

  if (days > 0) {
    return `${String(days)}d ${String(hours)}h`;
  }
  if (hours > 0) {
    return `${String(hours)}h ${String(minutes % 60)}m`;
  }

  return `${String(minutes)}m`;
}

function resetTime(window: Window, now: number) {
  const date = new Date(resetsAt(window, now));
  if (window.span === "month") {
    return date.toLocaleDateString(undefined, {
      day: "numeric",
      month: "long",
    });
  }

  return date.toLocaleString(undefined, {
    weekday: "long",
    hour: "numeric",
    minute: "2-digit",
  });
}
