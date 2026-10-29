import { useQuery } from "@tanstack/react-query";
import {
  CodeView,
  type CodeViewHandle,
  type CodeViewItem,
} from "@pierre/diffs/react";
import { useEffect, useMemo, useRef, useState } from "react";

import { Choice } from "@/components/choice";
import { TileStatus } from "@/components/tile-status";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { diffQuery } from "@/features/github/api/github";
import { DiffTally } from "@/features/github/components/diff-tally";
import { useReported } from "@/features/github/hooks/use-reported";
import {
  diffCss,
  hiddenReason,
  isGenerated,
  parsed,
  shownFile,
  versionOf,
} from "@/features/github/lib/diff";
import { diffThemes } from "@/features/github/lib/diff-theme";
import {
  arrive,
  setDiffScope,
  setDiffStyle,
  setShown,
  useDiffOptions,
  useTarget,
} from "@/features/github/lib/state";
import type { Diff } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";
import { useTheme } from "@/lib/theme";

const scopes = [
  { value: "uncommitted", label: "Uncommitted" },
  { value: "branch", label: "Branch" },
] as const;

const styles = [
  { value: "unified", label: "Unified" },
  { value: "split", label: "Split" },
] as const;

interface Props {
  machine: Machine;
  path: string;
  branch: string | null;
  uncommitted: number;
}

export function DiffView({ machine, path, branch, uncommitted }: Props) {
  const options = useDiffOptions();
  const scope = options.scope ?? (uncommitted > 0 ? "uncommitted" : "branch");
  const diff = useQuery(diffQuery(machine, path, scope));

  useReported("Could not refresh the diff", diff.data ? diff.error : null);

  return (
    <section
      aria-label="Diff"
      className="flex min-h-0 min-w-0 flex-1 flex-col gap-2"
    >
      <div className="flex shrink-0 items-center gap-4">
        <Choice
          label="Compare"
          value={scope}
          options={scopes}
          onChange={setDiffScope}
        />
        <div className="flex min-w-0 flex-1">
          {diff.data && (
            <DiffTally diff={diff.data} scope={scope} branch={branch} />
          )}
        </div>
        <Choice
          label="Layout"
          value={options.style}
          options={styles}
          onChange={setDiffStyle}
        />
      </div>
      {diff.isError && !diff.data ? (
        <TileStatus
          notice="Could not read the diff"
          retry="Try again"
          fill
          onRetry={() => {
            void diff.refetch();
          }}
        />
      ) : !diff.data ? (
        <TileStatus notice="Reading the diff…" busy fill />
      ) : diff.data.files.length === 0 ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>
              {scope === "uncommitted"
                ? "No uncommitted changes"
                : `No changes${diff.data.base ? ` since ${diff.data.base}` : ""}`}
            </EmptyTitle>
            {scope === "uncommitted" && (
              <EmptyDescription>
                Branch shows everything this branch adds.
              </EmptyDescription>
            )}
          </EmptyHeader>
        </Empty>
      ) : (
        <Files diff={diff.data} style={options.style} />
      )}
    </section>
  );
}

interface FilesProps {
  diff: Diff;
  style: "unified" | "split";
}

function Files({ diff, style }: FilesProps) {
  const theme = useTheme();
  const target = useTarget();
  const view = useRef<CodeViewHandle<undefined, undefined>>(null);
  const [opened, setOpened] = useState<ReadonlySet<string>>(new Set());
  const metadata = useMemo(
    () => diff.files.map((file) => ({ file, parsed: parsed(file) })),
    [diff],
  );
  const items = useMemo(
    () =>
      metadata.flatMap(({ file, parsed }): CodeViewItem<undefined>[] => {
        if (!parsed) {
          return [];
        }

        const collapsed = isGenerated(file.path) && !opened.has(file.path);

        return [
          {
            id: file.path,
            type: "diff",
            fileDiff: parsed,
            collapsed,
            version: versionOf(file, collapsed),
          },
        ];
      }),
    [metadata, opened],
  );
  const hidden = diff.files.filter((file) => hiddenReason(file) !== null);
  const options = useMemo(
    () => ({
      diffStyle: style,
      theme: diffThemes,
      themeType: theme,
      lineDiffType: "word" as const,
      overflow: "scroll" as const,
      tokenizeMaxLineLength: 1000,
      maxLineDiffLength: 5000,
      unsafeCSS: diffCss,
    }),
    [style, theme],
  );

  useEffect(() => {
    if (!target) {
      return;
    }

    if (items.some(({ id }) => id === target)) {
      view.current?.scrollTo({ type: "item", id: target, align: "start" });
    }
    arrive();
  }, [target, items]);

  useEffect(
    () => () => {
      setShown(undefined);
    },
    [],
  );

  return (
    <CodeView
      ref={view}
      items={items}
      options={options}
      className="diff min-h-0 flex-1 overflow-auto"
      onScroll={(top, viewer) => {
        const root = viewer.getContainerElement();
        const height = root?.clientHeight ?? 0;
        const remaining = root
          ? root.scrollHeight - height - root.scrollTop
          : 0;

        setShown(
          shownFile(
            items.map(({ id }) => id),
            (id) => viewer.getTopForItem(id),
            { top, height, remaining },
          ),
        );
      }}
      renderCodeViewHeader={() =>
        (hidden.length > 0 || diff.truncated) && (
          <ul className="flex flex-col gap-2 pb-2 type-caption text-muted-foreground">
            {hidden.map((file) => (
              <li key={file.path}>
                {file.path}: {hiddenReason(file)}
              </li>
            ))}
            {diff.truncated && (
              <li>Some files are left out to keep the view fast.</li>
            )}
          </ul>
        )
      }
      renderHeaderMetadata={(item) =>
        isGenerated(item.id) && (
          <Button
            size="xs"
            variant="ghost"
            onClick={() => {
              setOpened((current) => {
                const next = new Set(current);
                if (next.has(item.id)) {
                  next.delete(item.id);
                } else {
                  next.add(item.id);
                }

                return next;
              });
            }}
          >
            {opened.has(item.id)
              ? "Hide generated file"
              : "Show generated file"}
          </Button>
        )
      }
    />
  );
}
