import { ArrowCounterClockwiseIcon } from "@phosphor-icons/react";
import { useId } from "react";

import { Field, FieldLabel } from "@/components/ui/field";
import { Textarea } from "@/components/ui/textarea";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { ActionButton } from "@/features/github/components/action-button";
import { useWrite } from "@/features/github/hooks/use-write";
import { ago } from "@/features/github/lib/scope";
import { setDraft, useDraft } from "@/features/github/lib/state";
import type { CheckoutStatus, FileChange } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  checkout: CheckoutStatus;
  chosen: FileChange[];
}

export function CommitBox({ machine, checkout, chosen }: Props) {
  const id = useId();
  const commit = useWrite(machine, "Could not commit");
  const { path, files } = checkout;
  const draft = `${machine.id}:commit:${path}`;
  const message = useDraft(draft);
  const count =
    chosen.length === 1 ? "1 file" : `${String(chosen.length)} files`;

  return (
    <div className="flex flex-col gap-2">
      {files.length > 0 && (
        <>
          <Field>
            <FieldLabel htmlFor={`${id}-message`} className="sr-only">
              Commit message
            </FieldLabel>
            <Textarea
              id={`${id}-message`}
              value={message}
              placeholder="Commit message"
              rows={1}
              className="max-h-40 min-h-8 py-1"
              onChange={(event) => {
                setDraft(draft, event.target.value);
              }}
            />
          </Field>
          <ActionButton
            variant="default"
            label={`Commit ${count}`}
            busy="Committing…"
            pending={commit.isPending}
            disabled={chosen.length === 0 || message.trim() === ""}
            onClick={() => {
              const paths = chosen.flatMap((file) =>
                file.from ? [file.from, file.path] : [file.path],
              );
              commit.mutate(
                (client) =>
                  client.call("checkouts.commit", { path, paths, message }),
                {
                  onSuccess: () => {
                    setDraft(draft, "");
                  },
                },
              );
            }}
          />
        </>
      )}
      <LastCommit machine={machine} checkout={checkout} />
    </div>
  );
}

function LastCommit({ machine, checkout }: Omit<Props, "chosen">) {
  const undo = useWrite(machine, "Could not undo the commit");
  const { last, path } = checkout;
  if (!last?.undoable) {
    return null;
  }

  return (
    <div className="flex min-w-0 items-center gap-2">
      <Tooltip>
        <TooltipTrigger
          render={<span className="min-w-0 flex-1 truncate type-body" />}
        >
          {last.subject}
          <span className="sr-only">
            , committed {ago(last.date)}, not pushed yet
          </span>
        </TooltipTrigger>
        <TooltipContent className="flex-col items-start">
          <span>{last.subject}</span>
          <span>Committed {ago(last.date)}, not pushed yet</span>
        </TooltipContent>
      </Tooltip>
      <ActionButton
        variant="ghost"
        label="Undo"
        busy="Undoing…"
        icon={<ArrowCounterClockwiseIcon data-icon="inline-start" />}
        pending={undo.isPending}
        onClick={() => {
          undo.mutate((client) =>
            client.call("checkouts.undo", { path, sha: last.sha }),
          );
        }}
      />
    </div>
  );
}
