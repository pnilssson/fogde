import { useQuery } from "@tanstack/react-query";

import { Choice } from "@/components/choice";
import { Bar, Layer } from "@/components/frame";
import { TileStatus } from "@/components/tile-status";
import { Empty, EmptyHeader, EmptyTitle } from "@/components/ui/empty";
import { accountsQuery } from "@/features/github/api/accounts";
import {
  checkoutsQuery,
  diffQuery,
  listQuery,
} from "@/features/github/api/github";
import { DiffFiles } from "@/features/github/components/diff-view";
import { DiffTally } from "@/features/github/components/diff-tally";
import { PullCard } from "@/features/github/components/pull-card";
import { entriesOf, nowhere, pullFor } from "@/features/github/lib/scope";
import { select, setTab } from "@/features/github/lib/state";
import { setPanelScope, usePanel } from "@/features/github/lib/panel";
import type { DiffScope } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { openPage } from "@/lib/page";

const scopes = [
  { value: "turn", label: "Last turn" },
  { value: "uncommitted", label: "Uncommitted" },
  { value: "branch", label: "Branch" },
] as const satisfies readonly { value: DiffScope; label: string }[];

interface Props {
  machine: Machine;
  path: string;
  agent: string | null;
}

export function ChangesPanel({ machine, path, agent }: Props) {
  const scope = usePanel((panel) => panel.scope);

  return (
    <Layer aria-label="Changes" className="w-96 shrink-0 xl:w-1/3">
      <Bar>
        <Choice
          label="Changes"
          value={scope}
          options={scopes}
          onChange={setPanelScope}
        />
      </Bar>
      <Pull machine={machine} path={path} />
      <Changes machine={machine} path={path} scope={scope} agent={agent} />
    </Layer>
  );
}

interface ChangesProps {
  machine: Machine;
  path: string;
  scope: DiffScope;
  agent: string | null;
}

function Changes({ machine, path, scope, agent }: ChangesProps) {
  const waiting = scope === "turn" && !agent;
  const diff = useQuery({
    ...diffQuery(machine, path, scope, scope === "turn" ? agent : null),
    enabled: !waiting,
  });

  if (waiting) {
    return <Nothing title="Focus an agent to see its last turn" />;
  }
  if (diff.isError && !diff.data) {
    return (
      <TileStatus
        notice="Could not read the changes"
        retry="Try again"
        fill
        onRetry={() => {
          void diff.refetch();
        }}
      />
    );
  }
  if (!diff.data) {
    return <TileStatus notice="Reading the changes…" busy fill />;
  }
  if (diff.data.files.length === 0) {
    return <Nothing title={emptyTitle(scope, diff.data.base)} />;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2 px-2 pb-2">
      <DiffTally diff={diff.data} scope={scope} />
      <DiffFiles diff={diff.data} style="unified" follow={false} />
    </div>
  );
}

function Pull({ machine, path }: { machine: Machine; path: string }) {
  const checkouts = useQuery(checkoutsQuery(machine, [path]));
  const accounts = useQuery(accountsQuery(machine));
  const checkout = checkouts.data?.[0];
  const repo = checkout?.repo ?? undefined;
  const online =
    repo !== undefined &&
    accounts.data?.some(({ host }) => host === repo.host) === true;
  const open = useQuery({
    ...listQuery(machine, "pull", repo ?? nowhere, "open"),
    enabled: online,
  });
  const pull =
    checkout && online
      ? pullFor(entriesOf(repo, open.data), checkout)
      : undefined;
  if (!checkout?.branch || !pull) {
    return null;
  }

  return (
    <div className="px-2 pb-2">
      <PullCard
        branch={checkout.branch}
        pull={pull}
        canOpen={false}
        primary={false}
        onShow={(entry) => {
          select(machine.id, {
            kind: "pull",
            repo: entry.repo,
            number: entry.item.number,
          });
          setTab("pull");
          openPage("github", machine.id);
        }}
        onCreate={() => undefined}
      />
    </div>
  );
}

function Nothing({ title }: { title: string }) {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>{title}</EmptyTitle>
      </EmptyHeader>
    </Empty>
  );
}

function emptyTitle(scope: DiffScope, base: string | null) {
  if (scope === "turn") {
    return "No changes in the last turn";
  }
  if (scope === "uncommitted") {
    return "No uncommitted changes";
  }

  return `No changes${base ? ` since ${base}` : ""}`;
}
