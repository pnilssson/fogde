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
import { ActionButton } from "@/features/github/components/action-button";
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
  id: string;
  number: number;
  pull: PullDetail;
}

export function MergeButton({ machine, repo, id, number, pull }: Props) {
  const field = useId();
  const [open, setOpen] = useState(false);
  const [method, setMethod] = useState<MergeMethod | undefined>(
    pull.methods[0],
  );
  const [remove, setRemove] = useState(!pull.cross);
  const merge = useWrite(machine, "Could not merge");
  const auto = useWrite(machine, "Could not change auto-merge");
  const blocked = pull.merge === "conflicts" || pull.methods.length === 0;
  const later =
    pull.automerge && (pull.merge === "blocked" || pull.merge === "behind");
  const pending = merge.isPending || auto.isPending;

  if (pull.auto) {
    return (
      <ActionButton
        label="Cancel auto-merge"
        busy="Cancelling…"
        pending={auto.isPending}
        onClick={() => {
          auto.mutate((client) =>
            client.call("github.setAutoMerge", { repo, id, method: null }),
          );
        }}
      />
    );
  }

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
        {later ? "Merge when ready" : "Merge"}
      </Button>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {later
              ? `Merge #${String(number)} into ${pull.base} when ready?`
              : `Merge #${String(number)} into ${pull.base}?`}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {later
              ? "GitHub merges it as soon as the required reviews and checks pass. You can cancel until then."
              : "GitHub merges it right away, and it can only be undone with a revert."}
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
          {!pull.cross && !later && (
            <Field orientation="horizontal">
              <Checkbox
                id={`${field}-delete`}
                checked={remove}
                onCheckedChange={(checked) => {
                  setRemove(checked);
                }}
              />
              <FieldLabel htmlFor={`${field}-delete`}>
                Delete {pull.head} on GitHub
              </FieldLabel>
            </Field>
          )}
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel size="sm">Cancel</AlertDialogCancel>
          <AlertDialogAction
            size="sm"
            disabled={pending || !method}
            aria-busy={pending}
            onClick={() => {
              if (!method) {
                return;
              }

              const done = {
                onSettled: () => {
                  setOpen(false);
                },
              };
              if (later) {
                auto.mutate(
                  (client) =>
                    client.call("github.setAutoMerge", { repo, id, method }),
                  done,
                );
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
                done,
              );
            }}
          >
            {pending && <Spinner aria-hidden data-icon="inline-start" />}
            {later
              ? pending
                ? "Turning on…"
                : "Turn on auto-merge"
              : pending
                ? "Merging…"
                : "Merge"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
