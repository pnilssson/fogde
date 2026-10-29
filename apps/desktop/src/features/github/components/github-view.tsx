import { useQueries, useQuery } from "@tanstack/react-query";
import { type ReactNode, useMemo } from "react";

import { Body, Main } from "@/components/frame";
import { PageShell } from "@/components/shell";
import { PageBar } from "@/components/page-bar";
import { TileStatus } from "@/components/tile-status";
import { accountsQuery } from "@/features/github/api/accounts";
import { checkoutsQuery, listQuery } from "@/features/github/api/github";
import { DetailPane } from "@/features/github/components/detail-pane";
import { GitHubBar } from "@/features/github/components/github-bar";
import { SidePanel } from "@/features/github/components/side-panel";
import { useReported } from "@/features/github/hooks/use-reported";
import { foldersOf, merged, reposOf } from "@/features/github/lib/scope";
import {
  sameRepo,
  useChosen,
  useGitHubState,
} from "@/features/github/lib/state";
import type { Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspaces: Workspace[];
  current: Workspace | undefined;
  picker: ReactNode;
}

export function GitHubView({ machine, workspaces, current, picker }: Props) {
  const { scope, tab, closed } = useGitHubState();
  const chosen = useChosen(machine.id);
  const kind = tab === "issue" ? "issue" : "pull";
  const shown = useMemo(
    () => (scope === "workspace" && current ? [current] : workspaces),
    [scope, current, workspaces],
  );
  const checkouts = useQuery(checkoutsQuery(machine, foldersOf(shown)));
  const accounts = useQuery(accountsQuery(machine));
  const repos = useMemo(() => reposOf(checkouts.data ?? []), [checkouts.data]);
  const signedIn = repos.filter((repo) =>
    accounts.data?.some(({ host }) => host === repo.host),
  );
  const lists = useQueries({
    queries: signedIn.map((repo) => listQuery(machine, kind, repo, "open")),
  });
  const shut = useQueries({
    queries: signedIn.map((repo) => ({
      ...listQuery(machine, kind, repo, "closed"),
      enabled: closed && tab !== "changes",
    })),
  });
  const open = useQueries({
    queries: signedIn.map((repo) => listQuery(machine, "pull", repo, "open")),
  });

  const failed =
    [accounts, ...lists, ...shut, ...open].find(({ error }) => error)?.error ??
    null;
  useReported("Could not load GitHub", failed);
  useReported(
    "Could not read the checkouts",
    checkouts.data ? checkouts.error : null,
  );

  if (checkouts.isError && !checkouts.data) {
    return (
      <PageShell>
        <PageBar>
          <span className="flex-1" />
          {picker}
        </PageBar>
        <TileStatus
          notice="Could not read the checkouts"
          retry="Try again"
          fill
          onRetry={() => {
            void checkouts.refetch();
          }}
        />
      </PageShell>
    );
  }
  if (!checkouts.data) {
    return (
      <PageShell>
        <PageBar>
          <span className="flex-1" />
          {picker}
        </PageBar>
        <TileStatus notice="Reading checkouts…" busy fill />
      </PageShell>
    );
  }

  const entries = merged(
    signedIn.map((repo, index) => ({ repo, items: lists[index]?.data })),
  );
  const closedEntries = merged(
    signedIn.map((repo, index) => ({ repo, items: shut[index]?.data })),
  );
  const openPulls = merged(
    signedIn.map((repo, index) => ({ repo, items: open[index]?.data })),
  );
  const unsigned = accounts.data
    ? [
        ...new Set(
          repos
            .filter((repo) => !signedIn.includes(repo))
            .map(({ host }) => host),
        ),
      ]
    : [];
  const checkout =
    checkouts.data.find(({ path }) => path === chosen.checkout) ??
    checkouts.data[0];
  const online = signedIn.some(
    (repo) => checkout?.repo != null && sameRepo(repo, checkout.repo),
  );
  const item = tab === "issue" ? chosen.issue : chosen.pull;
  const where =
    scope === "workspace" && current ? current.name : "these workspaces";

  return (
    <PageShell
      side={
        <SidePanel
          machine={machine}
          tab={tab}
          closed={{
            shown: closed,
            entries: closedEntries,
            loading: shut.some(({ isPending }) => isPending),
          }}
          checkout={checkout}
          where={where}
          entries={entries}
          loading={lists.some(({ isPending }) => isPending)}
          chosen={item}
          signedIn={signedIn}
          unsigned={unsigned}
        />
      }
    >
      <GitHubBar
        machine={machine}
        current={current}
        scope={scope}
        checkouts={checkouts.data}
        checkout={checkout}
        openPulls={openPulls}
        online={online}
        picker={picker}
      />
      <Body>
        <Main>
          <DetailPane
            machine={machine}
            tab={tab}
            checkout={checkout}
            item={item}
            checkouts={checkouts.data}
            openPulls={openPulls}
            online={online}
          />
        </Main>
      </Body>
    </PageShell>
  );
}
