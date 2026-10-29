import { useQuery } from "@tanstack/react-query";
import { useId, useState } from "react";

import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { optionsQuery } from "@/features/github/api/github";
import { TextDialog } from "@/features/github/components/text-dialog";
import { useWrite } from "@/features/github/hooks/use-write";
import type { Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  repo: Repo;
  path: string;
  branch: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (number: number) => void;
}

export function NewPull({
  machine,
  repo,
  path,
  branch,
  open,
  onOpenChange,
  onCreated,
}: Props) {
  const id = useId();
  const options = useQuery({ ...optionsQuery(machine, repo), enabled: open });
  const create = useWrite<number>(machine, "Could not open the pull request");
  const [base, setBase] = useState<string>();
  const [draft, setDraft] = useState(false);
  const [missing, setMissing] = useState(false);
  const target = base ?? options.data?.branch ?? "";
  const loading = options.isPending && base === undefined;

  return (
    <TextDialog
      open={open}
      heading={`Open a pull request for ${branch}`}
      action="Open pull request"
      busy="Opening…"
      initial={{ title: title(branch), body: "" }}
      onOpenChange={onOpenChange}
      onSubmit={async ({ title, body }) => {
        if (loading || target.trim() === "") {
          setMissing(!loading);
          return;
        }

        const number = await create.mutateAsync((client) =>
          client.call("github.newPull", {
            path,
            repo,
            title,
            body,
            base: target.trim(),
            draft,
          }),
        );
        onOpenChange(false);
        onCreated(number);
      }}
    >
      <Field data-invalid={missing}>
        <FieldLabel htmlFor={`${id}-base`}>Merge into</FieldLabel>
        <Input
          id={`${id}-base`}
          value={target}
          disabled={loading}
          placeholder={loading ? "Loading the default branch…" : "main"}
          spellCheck={false}
          aria-invalid={missing}
          onChange={(event) => {
            setBase(event.target.value);
            setMissing(false);
          }}
        />
        {missing && <FieldError>Enter a branch</FieldError>}
      </Field>
      <Field orientation="horizontal">
        <Checkbox
          id={`${id}-draft`}
          checked={draft}
          onCheckedChange={(checked) => {
            setDraft(checked);
          }}
        />
        <FieldLabel htmlFor={`${id}-draft`}>Open as a draft</FieldLabel>
      </Field>
    </TextDialog>
  );
}

function title(branch: string) {
  const name = branch.split("/").at(-1) ?? branch;
  const words = name.replace(/[-_]+/g, " ").trim();

  return words.charAt(0).toUpperCase() + words.slice(1);
}
