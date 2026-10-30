import { CaretDownIcon, CaretRightIcon } from "@phosphor-icons/react";

import { TileStatus } from "@/components/tile-status";
import type { Kind } from "@/features/github/api/github";
import { ItemRow } from "@/features/github/components/rows";
import type { Entry } from "@/features/github/lib/scope";
import { type Item, sameItem, toggleClosed } from "@/features/github/lib/state";
import type { Filter, Repo, Section } from "@/generated/protocol";

const titles: Record<Section, string> = {
  review: "Needs your review",
  mine: "Yours",
  assigned: "Assigned to you",
  other: "Everything else",
};

const order: Section[] = ["review", "mine", "assigned", "other"];

export interface Closed {
  shown: boolean;
  entries: Entry[];
  loading: boolean;
}

interface Props {
  kind: Kind;
  entries: Entry[];
  closed: Closed;
  repo: Repo;
  loading: boolean;
  chosen: Item | undefined;
  current?: number;
  onSelect: (entry: Entry) => void;
}

export function ItemList({
  kind,
  entries,
  closed,
  repo,
  loading,
  chosen,
  current,
  onSelect,
}: Props) {
  if (loading && entries.length === 0) {
    return <TileStatus notice="Loading…" busy />;
  }

  const others = entries.filter(({ item }) => item.number !== current);
  const sections = [
    {
      key: "current",
      title: "This branch",
      shown: entries.filter(({ item }) => item.number === current),
    },
    ...order.map((section) => ({
      key: section,
      title: titles[section],
      shown: others.filter(({ item }) => item.section === section),
    })),
  ].filter(({ shown }) => shown.length > 0);
  const rows = (shown: Entry[]) =>
    shown.map((entry) => (
      <ItemRow
        key={`${entry.repo.owner}/${entry.repo.name}#${String(entry.item.number)}`}
        kind={kind}
        item={entry.item}
        active={sameItem(chosen, {
          kind,
          repo: entry.repo,
          number: entry.item.number,
        })}
        onSelect={() => {
          onSelect(entry);
        }}
      />
    ));

  return (
    <div className="flex flex-col gap-2">
      {entries.length === 0 ? (
        <Notice>{empty(kind, "open", repo)}</Notice>
      ) : (
        sections.map(({ key, title, shown }) => (
          <section key={key} aria-label={title} className="flex flex-col">
            <h3 className="flex h-8 items-center gap-2 px-2 type-caption text-muted-foreground">
              <span className="flex-1">{title}</span>
              <span>{shown.length}</span>
            </h3>
            {rows(shown)}
          </section>
        ))
      )}
      <section aria-label="Closed" className="flex flex-col">
        <h3>
          <button
            type="button"
            aria-expanded={closed.shown}
            className="flex h-8 w-full items-center gap-2 rounded-md px-2 type-caption text-muted-foreground outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring"
            onClick={toggleClosed}
          >
            <span className="flex-1 text-left">Closed</span>
            {closed.shown && !closed.loading && (
              <span>{closed.entries.length}</span>
            )}
            {closed.shown ? (
              <CaretDownIcon aria-hidden className="size-4" />
            ) : (
              <CaretRightIcon aria-hidden className="size-4" />
            )}
          </button>
        </h3>
        {closed.shown &&
          (closed.loading && closed.entries.length === 0 ? (
            <TileStatus notice="Loading…" busy />
          ) : closed.entries.length === 0 ? (
            <Notice>{empty(kind, "closed", repo)}</Notice>
          ) : (
            rows(closed.entries)
          ))}
      </section>
    </div>
  );
}

function Notice({ children }: { children: string }) {
  return <p className="px-2 type-caption text-muted-foreground">{children}</p>;
}

function empty(kind: Kind, filter: Filter, repo: Repo) {
  const what = kind === "pull" ? "pull requests" : "issues";

  return `No ${filter} ${what} in ${repo.owner}/${repo.name}.`;
}
