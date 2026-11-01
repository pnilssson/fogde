import { Progress } from "@base-ui/react/progress";

import { AgentIcon } from "@/components/agent-icon";
import { Button } from "@/components/ui/button";
import { ProgressIndicator, ProgressTrack } from "@/components/ui/progress";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useUsage } from "@/features/usage/hooks/use-usage";
import { useNow } from "@/hooks/use-now";
import type { Agent, Span, Window } from "@/generated/protocol";
import { agents } from "@/lib/agents";
import type { Machine } from "@/lib/machines";
import { countdown } from "@/lib/time";

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
    <ul className="flex flex-col" aria-label="Usage left">
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
            className="w-full justify-start px-2 text-muted-foreground"
            aria-label={`${agents[agent].short}: ${lines.join(", ")}`}
          />
        }
      >
        <AgentIcon agent={agent} />
        <Progress.Root value={left} aria-hidden className="flex-1">
          <ProgressTrack className="h-1">
            <ProgressIndicator className="rounded-full bg-muted-foreground" />
          </ProgressTrack>
        </Progress.Root>
        <span className="flex gap-2 font-mono type-caption tabular-nums">
          <span>{left}%</span>
          <span>{remaining(shown, now)}</span>
        </span>
      </TooltipTrigger>
      <TooltipContent side="right" className="max-w-none flex-col items-start">
        {lines.map((line) => (
          <span key={line}>{line}</span>
        ))}
      </TooltipContent>
    </Tooltip>
  );
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
  return countdown(resetsAt(window, now) - now);
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
