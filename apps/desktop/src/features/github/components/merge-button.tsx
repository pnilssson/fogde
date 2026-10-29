import { GitMergeIcon } from "@phosphor-icons/react";
import { useId, useState } from "react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldLabel } from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Spinner } from "@/components/ui/spinner";
import { useWrite } from "@/features/github/hooks/use-write";
import type { MergeMethod, PullDetail, Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const labels: Record<MergeMethod, string> = {
  squash: "Squash and merge",
  merge: "Create a merge commit",
  rebase: "Rebase and merge",
};

interface Props {
  machine: Machine;
  repo: Repo;
  number: number;
  pull: PullDetail;
}

export function MergeButton({ machine, repo, number, pull }: Props) {
  const id = useId();
  const [open, setOpen] = useState(false);
  const [method, setMethod] = useState<MergeMethod | undefined>(
    pull.methods[0],
  );
  const [remove, setRemove] = useState(!pull.cross);
  const merge = useWrite(machine, "Could not merge");
  const blocked = pull.merge === "conflicts" || pull.methods.length === 0;

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <Button
        size="sm"
        disabled={blocked}
        onClick={() => {
          setOpen(true);
        }}
      >
        <GitMergeIcon data-icon="inline-start" />
        Merge
      </Button>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{`Merge #${String(number)} into ${pull.base}?`}</AlertDialogTitle>
          <AlertDialogDescription>
            GitHub merges it right away, and it can only be undone with a
            revert.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <div className="flex flex-col gap-2">
          <Select
            items={pull.methods.map((value) => ({
              value,
              label: labels[value],
            }))}
            value={method}
            onValueChange={(next) => {
              const found = pull.methods.find((value) => value === next);
              if (found) {
                setMethod(found);
              }
            }}
          >
            <SelectTrigger
              size="sm"
              aria-label="Merge method"
              className="w-full"
            >
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {pull.methods.map((value) => (
                <SelectItem key={value} value={value}>
                  {labels[value]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {!pull.cross && (
            <Field orientation="horizontal">
              <Checkbox
                id={`${id}-delete`}
                checked={remove}
                onCheckedChange={(checked) => {
                  setRemove(checked);
                }}
              />
              <FieldLabel htmlFor={`${id}-delete`}>
                Delete {pull.head} on GitHub
              </FieldLabel>
            </Field>
          )}
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
          <AlertDialogAction
            size="sm"
            disabled={merge.isPending || !method}
            aria-busy={merge.isPending}
            onClick={() => {
              if (!method) {
                return;
              }

              merge.mutate(
                (client) =>
                  client.call("github.merge", {
                    repo,
                    number,
                    method,
                    sha: pull.sha,
                    delete: remove && !pull.cross,
                  }),
                {
                  onSettled: () => {
                    setOpen(false);
                  },
                },
              );
            }}
          >
            {merge.isPending && (
              <Spinner aria-hidden data-icon="inline-start" />
            )}
            {merge.isPending ? "Merging…" : "Merge"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
