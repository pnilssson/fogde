import {
  ArrowSquareOutIcon,
  PlayIcon,
  CaretRightIcon,
  CheckCircleIcon,
  CircleDashedIcon,
  MinusCircleIcon,
  XCircleIcon,
} from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";

import { TileStatus } from "@/components/tile-status";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { logQuery } from "@/features/github/api/github";
import { ActionButton } from "@/features/github/components/action-button";
import { useReported } from "@/features/github/hooks/use-reported";
import { useWrite } from "@/features/github/hooks/use-write";
import type { Check, CheckState, Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const order: Record<CheckState, number> = {
  failure: 0,
  pending: 1,
  success: 2,
  neutral: 3,
  skipped: 3,
};

interface Props {
  machine: Machine;
  repo: Repo;
  checks: Check[];
}

export function CheckList({ machine, repo, checks }: Props) {
  const rerun = useWrite(machine, "Could not re-run the checks");
  const failed = [
    ...new Set(
      checks
        .filter((check) => check.state === "failure" && check.run !== null)
        .map((check) => check.run),
    ),
  ].filter((run) => run !== null);
  const sorted = [...checks].sort(
    (a, b) =>
      order[a.state] - order[b.state] ||
      Number(b.required) - Number(a.required),
  );

  return (
    <div className="flex flex-col">
      <div className="flex items-center gap-2">
        <h3 className="flex-1 type-title">Checks</h3>
        {failed.length > 0 && (
          <ActionButton
            label="Re-run failed checks"
            busy="Re-running…"
            icon={<PlayIcon data-icon="inline-start" />}
            pending={rerun.isPending}
            onClick={() => {
              rerun.mutate(async (client) => {
                for (const run of failed) {
                  await client.call("github.rerun", { repo, run });
                }
              });
            }}
          />
        )}
      </div>
      <ul className="flex flex-col">
        {sorted.map((check, index) => (
          <li key={`${check.name}-${String(index)}`}>
            {check.state === "failure" && check.job !== null ? (
              <FailedCheck
                machine={machine}
                repo={repo}
                check={check}
                job={check.job}
              />
            ) : (
              <CheckRow check={check} />
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

function CheckRow({ check }: { check: Check }) {
  const { url } = check;
  if (!url) {
    return (
      <div className="flex h-8 min-w-0 items-center gap-2 px-2">
        <Name check={check} />
      </div>
    );
  }

  return (
    <button
      type="button"
      className="flex h-8 w-full min-w-0 items-center gap-2 rounded-md px-2 text-left outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring"
      onClick={() => void openUrl(url)}
    >
      <Name check={check} />
    </button>
  );
}

interface FailedProps {
  machine: Machine;
  repo: Repo;
  check: Check;
  job: number;
}

function FailedCheck({ machine, repo, check, job }: FailedProps) {
  const [open, setOpen] = useState(false);
  const log = useQuery({ ...logQuery(machine, repo, job), enabled: open });
  const { url } = check;

  useReported("Could not load the log", log.error);

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <div className="flex min-w-0 items-center gap-2">
        <CollapsibleTrigger className="group flex h-8 min-w-0 flex-1 items-center gap-2 rounded-md px-2 text-left outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring">
          <Name check={check} />
          <CaretRightIcon
            aria-hidden
            className="size-4 shrink-0 text-muted-foreground transition-transform ease-out group-data-[panel-open]:rotate-90 motion-reduce:transition-none"
          />
        </CollapsibleTrigger>
        {url && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={`Open ${check.name} on GitHub`}
            onClick={() => void openUrl(url)}
          >
            <ArrowSquareOutIcon />
          </Button>
        )}
      </div>
      <CollapsibleContent>
        {log.data ? (
          <div className="flex flex-col gap-2 py-2 pl-8">
            {log.data.step && (
              <p className="type-caption text-muted-foreground">
                Failed in {log.data.step}
              </p>
            )}
            <pre className="max-h-80 overflow-auto rounded-md bg-surface-2 p-2 type-code">
              {log.data.lines.join("\n")}
            </pre>
          </div>
        ) : (
          !log.isError && <TileStatus notice="Loading the log…" busy />
        )}
      </CollapsibleContent>
    </Collapsible>
  );
}

function Name({ check }: { check: Check }) {
  return (
    <>
      <StateIcon state={check.state} />
      <span className="min-w-0 flex-1 truncate type-body">{check.name}</span>
      {check.required && (
        <span className="shrink-0 type-caption text-muted-foreground">
          Required
        </span>
      )}
    </>
  );
}

function StateIcon({ state }: { state: CheckState }) {
  switch (state) {
    case "success":
      return (
        <CheckCircleIcon
          className="size-4 shrink-0 text-success"
          aria-label="Passed"
        />
      );
    case "failure":
      return (
        <XCircleIcon
          className="size-4 shrink-0 text-destructive"
          aria-label="Failed"
        />
      );
    case "pending":
      return (
        <CircleDashedIcon
          className="size-4 shrink-0 text-warning"
          aria-label="Running"
        />
      );
    case "skipped":
    case "neutral":
      return (
        <MinusCircleIcon
          className="size-4 shrink-0 text-muted-foreground"
          aria-label="Skipped"
        />
      );
  }
}
