import { ListIcon, TreeStructureIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import { useEffect, useId, useRef } from "react";

import { Choice } from "@/components/choice";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { moveFocus } from "@/features/github/lib/focus";
import {
  jump,
  setLayout,
  useLayout,
  useShown,
} from "@/features/github/lib/state";
import type { ChangeKind, FileChange } from "@/generated/protocol";

const layouts = [
  { value: "list", label: "List", icon: ListIcon },
  { value: "folders", label: "By folder", icon: TreeStructureIcon },
] as const;

interface Props {
  files: FileChange[];
  picked: ReadonlySet<string>;
  truncated: boolean;
  onPick: (paths: string[], on: boolean) => void;
}

export function ChangeList({ files, picked, truncated, onPick }: Props) {
  const id = useId();
  const layout = useLayout();
  const shown = useShown();
  const list = useRef<HTMLDivElement>(null);
  const chosen = files.filter((file) => picked.has(file.path)).length;
  const groups = layout === "folders" ? folders(files) : [["", files] as const];

  useEffect(() => {
    list.current
      ?.querySelector("[aria-current]")
      ?.scrollIntoView({ block: "nearest" });
  }, [shown]);

  return (
    <section aria-label="Changes" className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <Checkbox
          id={`${id}-all`}
          checked={chosen === files.length}
          indeterminate={chosen > 0 && chosen < files.length}
          onCheckedChange={(checked) => {
            onPick(
              files.map((file) => file.path),
              checked,
            );
          }}
        />
        <label htmlFor={`${id}-all`} className="flex-1 type-body">
          {chosen === files.length
            ? files.length === 1
              ? "1 changed file"
              : `${String(files.length)} changed files`
            : `${String(chosen)} of ${String(files.length)} selected`}
        </label>
        <Choice
          label="Group files"
          value={layout}
          options={layouts}
          onChange={setLayout}
        />
      </div>
      <div ref={list} className="flex flex-col gap-2" onKeyDown={moveFocus}>
        {groups.map(([folder, grouped]) => (
          <ul
            key={folder}
            className="flex flex-col"
            aria-label={folder || undefined}
          >
            {folder && (
              <li className="flex h-8 items-center gap-2">
                <Checkbox
                  aria-label={`Include ${folder}`}
                  checked={grouped.every((file) => picked.has(file.path))}
                  indeterminate={
                    grouped.some((file) => picked.has(file.path)) &&
                    !grouped.every((file) => picked.has(file.path))
                  }
                  onCheckedChange={(checked) => {
                    onPick(
                      grouped.map((file) => file.path),
                      checked,
                    );
                  }}
                />
                <span className="min-w-0 flex-1 truncate type-caption text-muted-foreground">
                  {folder}
                </span>
                <span className="type-caption text-muted-foreground">
                  {grouped.length}
                </span>
              </li>
            )}
            {grouped.map((file) => (
              <li key={file.path} className="flex min-w-0 items-center gap-2">
                <Checkbox
                  aria-label={`Include ${file.path}`}
                  checked={picked.has(file.path)}
                  onCheckedChange={(checked) => {
                    onPick([file.path], checked);
                  }}
                />
                <button
                  type="button"
                  data-row
                  aria-current={file.path === shown || undefined}
                  className={cn(
                    "flex h-8 min-w-0 flex-1 items-center gap-2 rounded-md px-2 text-left outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-ring",
                    file.path === shown && "bg-selected",
                  )}
                  onClick={() => {
                    jump(file.path);
                  }}
                >
                  <Name file={file} folder={folder} />
                  <Status change={file.change} />
                </button>
              </li>
            ))}
          </ul>
        ))}
      </div>
      {truncated && (
        <p className="type-caption text-muted-foreground">
          Only the first {files.length} changed files are listed. Commit the
          rest in a terminal.
        </p>
      )}
    </section>
  );
}

function Name({ file, folder }: { file: FileChange; folder: string }) {
  const name = split(file.path);
  const from = file.from ? split(file.from) : null;
  const dir = folder ? "" : name.dir;

  return (
    <span className="flex min-w-0 flex-1 items-baseline gap-2">
      <span
        className={cn(
          "shrink-0 truncate type-body",
          file.change === "deleted" && "line-through",
        )}
      >
        {from && from.base !== name.base
          ? `${from.base} to ${name.base}`
          : name.base}
      </span>
      {dir && (
        <span className="min-w-0 truncate type-caption text-muted-foreground">
          {dir}
        </span>
      )}
    </span>
  );
}

const marks: Record<
  ChangeKind,
  { letter: string; label: string; tone: string }
> = {
  added: { letter: "A", label: "Added", tone: "text-success" },
  untracked: { letter: "U", label: "Untracked", tone: "text-success" },
  modified: { letter: "M", label: "Modified", tone: "text-warning" },
  deleted: { letter: "D", label: "Deleted", tone: "text-destructive" },
  renamed: { letter: "R", label: "Renamed", tone: "text-muted-foreground" },
  copied: { letter: "C", label: "Copied", tone: "text-muted-foreground" },
  conflicted: { letter: "!", label: "Conflict", tone: "text-destructive" },
};

function Status({ change }: { change: ChangeKind }) {
  const { letter, label, tone } = marks[change];

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <span
            className={cn(
              "w-4 shrink-0 text-center font-mono type-caption",
              tone,
            )}
          />
        }
      >
        <span aria-hidden>{letter}</span>
        <span className="sr-only">{label}</span>
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

function split(path: string) {
  const at = path.lastIndexOf("/");

  return at < 0
    ? { dir: "", base: path }
    : { dir: path.slice(0, at), base: path.slice(at + 1) };
}

function folders(files: FileChange[]) {
  const groups = new Map<string, FileChange[]>();
  for (const file of files) {
    const { dir } = split(file.path);
    const key = dir || "/";
    groups.set(key, [...(groups.get(key) ?? []), file]);
  }

  return [...groups.entries()].sort(([a], [b]) => a.localeCompare(b));
}
