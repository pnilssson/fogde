import { useQuery } from "@tanstack/react-query";
import { Fragment } from "react";

import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { Changes, Stats } from "@/generated/protocol";
import { statsQuery } from "@/lib/stats";

interface Count {
  text: string;
  tone?: string;
}

interface Side {
  sign: string;
  count: number;
  tone?: string;
}

interface Part {
  key: keyof Stats;
  counts: Count[];
  sentence: string;
}

interface Props {
  changes: Changes | null | undefined;
}

function counted(number: number, one: string, many: string) {
  return `${String(number)} ${number === 1 ? one : many}`;
}

function signed(...sides: Side[]): Count[] {
  return sides
    .filter(({ count }) => count > 0)
    .map(({ sign, count, tone }) => ({
      text: `${sign}${String(count)}`,
      tone,
    }));
}

function partsOf(changes: Changes, stats: Stats) {
  const { added, removed, base, upstream } = changes;
  const parts: Part[] = [];

  if (stats.behind && base && base.behind > 0) {
    parts.push({
      key: "behind",
      counts: [{ text: `↓${String(base.behind)} ${base.name}` }],
      sentence: `${counted(base.behind, "commit", "commits")} behind ${base.name}`,
    });
  }

  if (stats.lines && (added > 0 || removed > 0)) {
    const since = base ? `since ${base.name}` : "since the last commit";
    parts.push({
      key: "lines",
      counts: signed(
        { sign: "+", count: added, tone: "text-success" },
        { sign: "−", count: removed, tone: "text-destructive" },
      ),
      sentence: `${counted(added, "line", "lines")} added and ${String(removed)} removed ${since}`,
    });
  }

  if (
    stats.upstream &&
    upstream &&
    (upstream.ahead > 0 || upstream.behind > 0)
  ) {
    parts.push({
      key: "upstream",
      counts: signed(
        { sign: "↑", count: upstream.ahead },
        { sign: "↓", count: upstream.behind },
      ),
      sentence: `${counted(upstream.ahead, "commit", "commits")} to push and ${String(upstream.behind)} to pull`,
    });
  }

  return parts;
}

export function BranchStats({ changes }: Props) {
  const { data } = useQuery(statsQuery);
  const parts = changes && data ? partsOf(changes, data) : [];
  if (parts.length === 0) {
    return null;
  }

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <span className="flex shrink-0 items-center gap-2 font-mono type-caption tabular-nums" />
        }
      >
        {parts.map(({ key, counts }) => (
          <span key={key}>
            {counts.map(({ text, tone }, index) => (
              <Fragment key={text}>
                {index > 0 && " "}
                <span className={tone}>{text}</span>
              </Fragment>
            ))}
          </span>
        ))}
      </TooltipTrigger>
      <TooltipContent className="max-w-none flex-col items-start">
        {parts.map(({ key, sentence }) => (
          <span key={key}>{sentence}</span>
        ))}
      </TooltipContent>
    </Tooltip>
  );
}
