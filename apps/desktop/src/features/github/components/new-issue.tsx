import { useState } from "react";

import { Field, FieldLabel } from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { TextDialog } from "@/features/github/components/text-dialog";
import { useWrite } from "@/features/github/hooks/use-write";
import type { Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  repos: Repo[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (repo: Repo, number: number) => void;
}

const key = (repo: Repo) => `${repo.host}/${repo.owner}/${repo.name}`;

export function NewIssue({
  machine,
  repos,
  open,
  onOpenChange,
  onCreated,
}: Props) {
  const create = useWrite<number>(machine, "Could not create the issue");
  const [chosen, setChosen] = useState<string>();
  const repo = repos.find((candidate) => key(candidate) === chosen) ?? repos[0];

  return (
    <TextDialog
      open={open}
      heading="New issue"
      action="Create"
      busy="Creating…"
      onOpenChange={onOpenChange}
      onSubmit={async ({ title, body }) => {
        if (!repo) {
          return;
        }

        const number = await create.mutateAsync((client) =>
          client.call("github.newIssue", { repo, title, body }),
        );
        onOpenChange(false);
        onCreated(repo, number);
      }}
    >
      {repos.length > 1 && repo && (
        <Field>
          <FieldLabel>Repository</FieldLabel>
          <Select
            items={repos.map((candidate) => ({
              value: key(candidate),
              label: `${candidate.owner}/${candidate.name}`,
            }))}
            value={key(repo)}
            onValueChange={(next) => {
              if (next) {
                setChosen(next);
              }
            }}
          >
            <SelectTrigger size="sm" aria-label="Repository" className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {repos.map((candidate) => (
                <SelectItem key={key(candidate)} value={key(candidate)}>
                  {candidate.owner}/{candidate.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
      )}
    </TextDialog>
  );
}
