import { GitBranchIcon, TreeStructureIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { cn } from "cn";
import { type ReactNode, useId, useMemo, useState } from "react";

import { Layer, SideScroll } from "@/components/frame";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Toggle } from "@/components/ui/toggle";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { diffQuery } from "@/features/github/api/github";
import { ChangeList } from "@/features/github/components/change-list";
import { CommitBox } from "@/features/github/components/commit-box";
import { NewBranch } from "@/features/github/components/new-branch";
import { usePlan } from "@/features/github/hooks/use-plan";
import {
  setLayout,
  showScope,
  useDiffScope,
  useLayout,
} from "@/features/github/lib/state";
import type { CheckoutStatus, DiffScope } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  checkout: CheckoutStatus;
  header: ReactNode;
  online: boolean;
}

export function ChangesPane({ machine, checkout, header, online }: Props) {
  const id = useId();
  const files = useMemo(
    () => [...checkout.files].sort((a, b) => a.path.localeCompare(b.path)),
    [checkout.files],
  );
  const scope = useDiffScope(files.length);
  const branch = useQuery(diffQuery(machine, checkout.path, "branch"));
  const [skipped, setSkipped] = useState<ReadonlySet<string>>(new Set());
  const [branching, setBranching] = useState(false);
  const { base } = usePlan(machine, checkout, false, online);
  const based = checkout.branch === null || checkout.branch === base;
  const chosen = files.filter((file) => !skipped.has(file.path));
  const picked = new Set(chosen.map((file) => file.path));
  const committed = branch.data?.files ?? [];
  const pick = (paths: string[], on: boolean) => {
    setSkipped((current) => {
      const next = new Set(current);
      for (const path of paths) {
        if (on) {
          next.delete(path);
        } else {
          next.add(path);
        }
      }

      return next;
    });
  };

  return (
    <>
      <Layer className="flex-1">
        {header}
        <SideScroll>
          <div className="flex flex-col gap-4">
            {based && (
              <Button
                variant="ghost"
                size="sm"
                className="justify-start"
                onClick={() => {
                  setBranching(true);
                }}
              >
                <GitBranchIcon data-icon="inline-start" />
                New branch
              </Button>
            )}
            <section aria-labelledby={`${id}-uncommitted`}>
              <div className="flex items-center gap-2">
                {files.length > 0 && (
                  <Checkbox
                    aria-label="Include every file"
                    checked={chosen.length === files.length}
                    indeterminate={
                      chosen.length > 0 && chosen.length < files.length
                    }
                    onCheckedChange={(checked) => {
                      pick(
                        files.map((file) => file.path),
                        checked,
                      );
                    }}
                  />
                )}
                <Heading
                  id={`${id}-uncommitted`}
                  scope="uncommitted"
                  active={scope === "uncommitted"}
                  disabled={files.length === 0}
                  note={
                    files.length === 0
                      ? "None"
                      : chosen.length === files.length
                        ? String(files.length)
                        : `${String(chosen.length)} of ${String(files.length)}`
                  }
                >
                  Uncommitted
                </Heading>
                <GroupToggle />
              </div>
              {files.length > 0 && (
                <ChangeList
                  label="Uncommitted files"
                  files={files}
                  active={scope === "uncommitted"}
                  picked={picked}
                  onPick={pick}
                  onShow={(path) => {
                    showScope("uncommitted", path);
                  }}
                />
              )}
              {checkout.truncated && (
                <p className="px-2 type-caption text-muted-foreground">
                  Only the first {files.length} changed files are listed. Commit
                  the rest in a terminal.
                </p>
              )}
            </section>
            {committed.length > 0 && (
              <section aria-labelledby={`${id}-branch`}>
                <Heading
                  id={`${id}-branch`}
                  scope="branch"
                  active={scope === "branch"}
                  note={String(committed.length)}
                >
                  On this branch
                </Heading>
                <ChangeList
                  label="Files changed on this branch"
                  files={committed}
                  active={scope === "branch"}
                  onShow={(path) => {
                    showScope("branch", path);
                  }}
                />
              </section>
            )}
          </div>
        </SideScroll>
      </Layer>
      <NewBranch
        machine={machine}
        path={checkout.path}
        from={checkout.branch}
        open={branching}
        onOpenChange={setBranching}
      />
      {(files.length > 0 || checkout.last?.undoable) && (
        <Layer aria-label="Commit" className="shrink-0 p-2">
          <CommitBox machine={machine} checkout={checkout} chosen={chosen} />
        </Layer>
      )}
    </>
  );
}

function GroupToggle() {
  const layout = useLayout();

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Toggle
            size="sm"
            aria-label="Group by folder"
            pressed={layout === "folders"}
            className="text-muted-foreground hover:bg-hover hover:text-foreground aria-pressed:bg-surface-3 aria-pressed:text-foreground aria-pressed:shadow-surface-2"
            onPressedChange={(pressed) => {
              setLayout(pressed ? "folders" : "list");
            }}
          />
        }
      >
        <TreeStructureIcon />
      </TooltipTrigger>
      <TooltipContent>Group by folder</TooltipContent>
    </Tooltip>
  );
}

interface HeadingProps {
  id: string;
  scope: DiffScope;
  active: boolean;
  disabled?: boolean;
  note: string;
  children: string;
}

function Heading({
  id,
  scope,
  active,
  disabled = false,
  note,
  children,
}: HeadingProps) {
  return (
    <h3 className="flex min-w-0 flex-1">
      <button
        id={id}
        type="button"
        aria-pressed={active}
        disabled={disabled}
        className={cn(
          "flex h-8 min-w-0 flex-1 items-center gap-2 rounded-md px-2 text-left type-caption text-muted-foreground outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring disabled:hover:bg-transparent",
          active && "text-foreground",
        )}
        onClick={() => {
          showScope(scope);
        }}
      >
        <span className="flex-1 truncate">{children}</span>
        <span className="shrink-0">{note}</span>
      </button>
    </h3>
  );
}
