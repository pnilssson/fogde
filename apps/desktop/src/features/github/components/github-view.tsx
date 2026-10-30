import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";

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
import {
  entriesOf,
  grouped,
  pathsOf,
  samePath,
} from "@/features/github/lib/scope";
import {
  sameRepo,
  useChosen,
  useGitHubState,
} from "@/features/github/lib/state";
import type { Repo, Workspace } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  workspaces: Workspace[];
  wanted: string | undefined;
  picker: ReactNode;
}

const nowhere: Repo = { host: "", owner: "", name: "" };

export function GitHubView({ machine, workspaces, wanted, picker }: Props) {
  const { tab, closed } = useGitHubState();
  const chosen = useChosen(machine.id);
  const kind = tab === "issue" ? "issue" : "pull";
  const checkouts = useQuery(checkoutsQuery(machine, pathsOf(workspaces)));
  const accounts = useQuery(accountsQuery(machine));
  const groups = grouped(workspaces, checkouts.data ?? []);
  const checkout =
    checkouts.data?.find(({ path }) => wanted && samePath(path, wanted)) ??
    checkouts.data?.[0];
  const known = checkout?.repo ?? undefined;
  const repo = accounts.data?.some(({ host }) => host === known?.host)
    ? known
    : undefined;
  const online = repo !== undefined;
  const shut = useQuery({
    ...listQuery(machine, kind, repo ?? nowhere, "closed"),
    enabled: online && closed && tab !== "changes",
  });
  const merged = useQuery({
    ...listQuery(machine, "pull", repo ?? nowhere, "closed"),
    enabled: online,
  });
  const issues = useQuery({
    ...listQuery(machine, "issue", repo ?? nowhere, "open"),
    enabled: online,
  });
  const open = useQuery({
    ...listQuery(machine, "pull", repo ?? nowhere, "open"),
    enabled: online,
  });
  const list = kind === "issue" ? issues : open;

  useReported(
    "Could not load GitHub",
    [accounts, shut, merged, issues, open].find(({ error }) => error)?.error ??
      null,
  );
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

  const openPulls = entriesOf(repo, open.data);
  const unsigned = known && accounts.data && !online ? known.host : undefined;
  const picked = tab === "issue" ? chosen.issue : chosen.pull;
  const item =
    picked && repo && sameRepo(picked.repo, repo) ? picked : undefined;

  return (
    <PageShell
      side={
        <SidePanel
          machine={machine}
          tab={tab}
          closed={{
            shown: closed,
            entries: entriesOf(repo, shut.data),
            loading: shut.isPending,
          }}
          checkout={checkout}
          entries={entriesOf(repo, list.data)}
          loading={list.isPending}
          chosen={item}
          repo={repo}
          unsigned={unsigned}
          counts={{
            changes: checkout
              ? `${String(checkout.files.length)}${checkout.truncated ? "+" : ""}`
              : undefined,
            pull: open.data?.length.toString(),
            issue: issues.data?.length.toString(),
          }}
        />
      }
    >
      <GitHubBar
        machine={machine}
        groups={groups}
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
            closedPulls={entriesOf(repo, merged.data)}
            linked={
              checkout !== undefined &&
              !workspaces.some(({ checkouts: listed }) =>
                listed.some(
                  ({ main, path }) => main && samePath(path, checkout.path),
                ),
              )
            }
            online={online}
          />
        </Main>
      </Body>
    </PageShell>
  );
}
