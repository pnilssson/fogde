import { PlusIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { Bar, Layer, SideScroll } from "@/components/frame";
import { IconAction } from "@/components/icon-action";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ChangesPane } from "@/features/github/components/changes-pane";
import { type Closed, ItemList } from "@/features/github/components/item-list";
import { NewIssue } from "@/features/github/components/new-issue";
import { SignInNotice } from "@/features/github/components/sign-in-notice";
import { moveFocus } from "@/features/github/lib/focus";
import type { Entry } from "@/features/github/lib/scope";
import {
  type Item,
  select,
  setTab,
  type Tab,
} from "@/features/github/lib/state";
import type { CheckoutStatus, Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  tab: Tab;
  closed: Closed;
  checkout: CheckoutStatus | undefined;
  where: string;
  entries: Entry[];
  loading: boolean;
  chosen: Item | undefined;
  signedIn: Repo[];
  unsigned: string[];
}

export function SidePanel({
  machine,
  tab,
  closed,
  checkout,
  where,
  entries,
  loading,
  chosen,
  signedIn,
  unsigned,
}: Props) {
  const [creating, setCreating] = useState(false);
  const tabs = (
    <Bar>
      <Tabs
        value={tab}
        className="min-w-0 gap-0"
        onValueChange={(next: Tab) => {
          setTab(next);
        }}
      >
        <TabsList aria-label="Show">
          <TabsTrigger value="changes">Changes</TabsTrigger>
          <TabsTrigger value="pull">PRs</TabsTrigger>
          <TabsTrigger value="issue">Issues</TabsTrigger>
        </TabsList>
      </Tabs>
      <span className="flex-1" />
      {tab === "issue" && signedIn.length > 0 && (
        <IconAction
          label="New issue"
          icon={<PlusIcon />}
          onClick={() => {
            setCreating(true);
          }}
        />
      )}
    </Bar>
  );

  if (tab === "changes" && checkout) {
    return (
      <ChangesPane
        key={checkout.path}
        machine={machine}
        checkout={checkout}
        header={tabs}
      />
    );
  }

  return (
    <Layer className="flex-1">
      {tabs}
      {tab === "changes" ? (
        <p className="px-4 pb-2 type-caption text-muted-foreground">
          No Git repositories in {where}.
        </p>
      ) : (
        <>
          <SideScroll>
            <div className="flex flex-col gap-2" onKeyDown={moveFocus}>
              {unsigned.map((host) => (
                <SignInNotice key={host} machine={machine} host={host} />
              ))}
              {signedIn.length === 0 && unsigned.length === 0 ? (
                <p className="px-2 type-caption text-muted-foreground">
                  No GitHub repositories in {where}.
                </p>
              ) : (
                signedIn.length > 0 && (
                  <ItemList
                    kind={tab}
                    closed={closed}
                    entries={entries}
                    repos={signedIn}
                    loading={loading}
                    chosen={chosen}
                    onSelect={({ repo, item }) => {
                      select(machine.id, {
                        kind: tab,
                        repo,
                        number: item.number,
                      });
                    }}
                  />
                )
              )}
            </div>
          </SideScroll>
          <NewIssue
            machine={machine}
            repos={signedIn}
            open={creating}
            onOpenChange={setCreating}
            onCreated={(repo, number) => {
              select(machine.id, { kind: "issue", repo, number });
            }}
          />
        </>
      )}
    </Layer>
  );
}
