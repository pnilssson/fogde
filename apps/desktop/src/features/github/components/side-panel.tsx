import {
  GitPullRequestIcon,
  PlusIcon,
  PlusMinusIcon,
  RecordIcon,
} from "@phosphor-icons/react";
import { useState } from "react";

import { Bar, Layer, SideScroll } from "@/components/frame";
import { TileStatus } from "@/components/tile-status";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { ChangesPane } from "@/features/github/components/changes-pane";
import { type Closed, ItemList } from "@/features/github/components/item-list";
import { NewIssue } from "@/features/github/components/new-issue";
import { PullStart } from "@/features/github/components/pull-start";
import { SignInNotice } from "@/features/github/components/sign-in-notice";
import { moveFocus } from "@/features/github/lib/focus";
import { type Entry, pullFor } from "@/features/github/lib/scope";
import {
  type Item,
  select,
  setTab,
  type Tab,
} from "@/features/github/lib/state";
import type { CheckoutStatus, Repo } from "@/generated/protocol";
import { folderName } from "@/lib/folders";
import type { Machine } from "@/lib/machines";

const views = [
  { value: "changes", label: "Changes", icon: PlusMinusIcon },
  { value: "pull", label: "Pull requests", icon: GitPullRequestIcon },
  { value: "issue", label: "Issues", icon: RecordIcon },
] as const;

interface Props {
  machine: Machine;
  tab: Tab;
  closed: Closed;
  checkout: CheckoutStatus | undefined;
  entries: Entry[];
  loading: boolean;
  chosen: Item | undefined;
  repo: Repo | undefined;
  unsigned: string | undefined;
  counts: Partial<Record<Tab, string>>;
}

export function SidePanel({
  machine,
  tab,
  closed,
  checkout,
  entries,
  loading,
  chosen,
  repo,
  unsigned,
  counts,
}: Props) {
  const [creating, setCreating] = useState(false);
  const current =
    tab === "pull" && checkout ? pullFor(entries, checkout) : undefined;
  const tabs = (
    <Bar>
      <Tabs
        value={tab}
        className="min-w-0 flex-1 gap-0"
        onValueChange={(next: Tab) => {
          setTab(next);
        }}
      >
        <TabsList aria-label="Show" className="w-full">
          {views.map(({ value, label, icon: Icon }) => {
            const count = counts[value];
            const shown = count !== undefined && count !== "0";

            return (
              <Tooltip key={value}>
                <TooltipTrigger
                  render={
                    <TabsTrigger
                      value={value}
                      aria-label={shown ? `${label}, ${count}` : label}
                    />
                  }
                >
                  <Icon />
                  {shown && <span className="tabular-nums">{count}</span>}
                </TooltipTrigger>
                <TooltipContent>{label}</TooltipContent>
              </Tooltip>
            );
          })}
        </TabsList>
      </Tabs>
    </Bar>
  );

  if (tab === "changes" && checkout) {
    return (
      <ChangesPane
        key={checkout.path}
        machine={machine}
        checkout={checkout}
        header={tabs}
        online={repo !== undefined}
      />
    );
  }

  return (
    <Layer className="flex-1">
      {tabs}
      {!checkout ? (
        <p className="px-4 pb-2 type-caption text-muted-foreground">
          No Git checkouts in your workspaces.
        </p>
      ) : (
        <>
          <SideScroll>
            <div className="flex flex-col gap-2" onKeyDown={moveFocus}>
              {unsigned && <SignInNotice machine={machine} host={unsigned} />}
              {tab === "pull" && repo && !loading && !current && (
                <PullStart machine={machine} checkout={checkout} repo={repo} />
              )}
              {tab === "issue" && repo && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="justify-start"
                  onClick={() => {
                    setCreating(true);
                  }}
                >
                  <PlusIcon data-icon="inline-start" />
                  New issue
                </Button>
              )}
              {repo ? (
                <ItemList
                  kind={tab === "issue" ? "issue" : "pull"}
                  closed={closed}
                  entries={entries}
                  repo={repo}
                  loading={loading}
                  chosen={chosen}
                  current={current?.item.number}
                  onSelect={({ repo: base, item }) => {
                    select(machine.id, {
                      kind: tab === "issue" ? "issue" : "pull",
                      repo: base,
                      number: item.number,
                    });
                  }}
                />
              ) : checkout.repo === null ? (
                <p className="px-2 type-caption text-muted-foreground">
                  {folderName(checkout.path)} has no GitHub remote.
                </p>
              ) : (
                !unsigned && <TileStatus notice="Loading…" busy />
              )}
            </div>
          </SideScroll>
          {repo && (
            <NewIssue
              machine={machine}
              repo={repo}
              open={creating}
              onOpenChange={setCreating}
              onCreated={(number) => {
                select(machine.id, { kind: "issue", repo, number });
              }}
            />
          )}
        </>
      )}
    </Layer>
  );
}
