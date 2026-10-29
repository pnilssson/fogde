import { CaretDownIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Spinner } from "@/components/ui/spinner";
import { optionsQuery } from "@/features/github/api/github";
import type { Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  repo: Repo;
  label: string;
  source: "labels" | "people";
  chosen: string[];
  onChange: (
    next: string[],
    added: string[],
    removed: string[],
  ) => Promise<unknown>;
}

export function PickMenu({
  machine,
  repo,
  label,
  source,
  chosen,
  onChange,
}: Props) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<string[]>(chosen);
  const [saving, setSaving] = useState(false);
  const options = useQuery({ ...optionsQuery(machine, repo), enabled: open });
  const names =
    source === "labels"
      ? (options.data?.labels ?? []).map(({ name }) => name)
      : (options.data?.people ?? []);
  const shown = [...new Set([...chosen, ...names])];

  return (
    <DropdownMenu
      open={open}
      onOpenChange={(next, details) => {
        setOpen(next);
        if (next) {
          setDraft(chosen);
          return;
        }
        if (details.reason === "escape-key") {
          return;
        }

        const added = draft.filter((name) => !chosen.includes(name));
        const removed = chosen.filter((name) => !draft.includes(name));
        if (added.length === 0 && removed.length === 0) {
          return;
        }

        setSaving(true);
        void onChange(draft, added, removed)
          .catch(() => undefined)
          .finally(() => {
            setSaving(false);
          });
      }}
    >
      <DropdownMenuTrigger
        render={
          <Button
            size="sm"
            variant="ghost"
            disabled={saving}
            aria-busy={saving}
          />
        }
      >
        {saving && <Spinner aria-hidden data-icon="inline-start" />}
        {saving ? "Saving…" : label}
        {!saving && <CaretDownIcon data-icon="inline-end" />}
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="max-h-80 w-auto min-w-48">
        {options.isPending && (
          <div className="flex items-center gap-2 p-2">
            <Spinner aria-hidden />
            <span className="type-caption">Loading…</span>
          </div>
        )}
        {shown.map((name) => (
          <DropdownMenuCheckboxItem
            key={name}
            closeOnClick={false}
            checked={draft.includes(name)}
            onCheckedChange={(checked) => {
              setDraft((current) =>
                checked
                  ? [...current, name]
                  : current.filter((other) => other !== name),
              );
            }}
          >
            {name}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
