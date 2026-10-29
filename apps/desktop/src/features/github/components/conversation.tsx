import { useId, useState } from "react";

import { Field, FieldLabel } from "@/components/ui/field";
import { Textarea } from "@/components/ui/textarea";
import type { Kind } from "@/features/github/api/github";
import { ActionButton } from "@/features/github/components/action-button";
import { RenderedHtml } from "@/features/github/components/rendered-html";
import { useWrite } from "@/features/github/hooks/use-write";
import { ago } from "@/features/github/lib/scope";
import { setDraft, useDraft } from "@/features/github/lib/state";
import type {
  Comment,
  Detail,
  Repo,
  ReviewEvent,
  Verdict,
} from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

const verdicts: Record<Verdict, string> = {
  approved: "approved",
  changes: "requested changes",
  commented: "reviewed",
  dismissed: "had a review dismissed",
};

interface Props {
  machine: Machine;
  kind: Kind;
  repo: Repo;
  detail: Detail;
}

export function Conversation({ machine, kind, repo, detail }: Props) {
  const id = useId();
  const draft = `${machine.id}:${repo.host}/${repo.owner}/${repo.name}#${String(detail.number)}`;
  const text = useDraft(draft);
  const comment = useWrite(machine, "Could not comment");
  const review = useWrite(machine, "Could not submit the review");
  const [event, setEvent] = useState<ReviewEvent>();
  const { number } = detail;
  const empty = text.trim() === "";
  const done = {
    onSuccess: () => {
      setDraft(draft, "");
    },
  };
  const submit = (next: ReviewEvent) => {
    setEvent(next);
    review.mutate(
      (client) =>
        client.call("github.review", { repo, number, event: next, body: text }),
      done,
    );
  };

  return (
    <section aria-label="Conversation" className="flex flex-col gap-4">
      {detail.linked.length > 0 && (
        <p className="type-caption text-muted-foreground">
          {kind === "pull" ? "Closes " : "Closed by "}
          {detail.linked
            .map((link) => `#${String(link.number)} ${link.title}`)
            .join(", ")}
        </p>
      )}
      {detail.comments.map((entry) => (
        <CommentEntry key={entry.id} comment={entry} />
      ))}
      <Field>
        <FieldLabel htmlFor={`${id}-comment`}>
          {kind === "pull" ? "Comment or review" : "Comment"}
        </FieldLabel>
        <Textarea
          id={`${id}-comment`}
          value={text}
          className="min-h-20"
          onChange={(change) => {
            setDraft(draft, change.target.value);
          }}
        />
      </Field>
      <div className="flex flex-wrap gap-2">
        <ActionButton
          variant="default"
          label="Comment"
          busy="Commenting…"
          pending={comment.isPending}
          disabled={empty || review.isPending}
          onClick={() => {
            comment.mutate(
              (client) =>
                client.call("github.comment", { repo, number, body: text }),
              done,
            );
          }}
        />
        {kind === "pull" && detail.state === "open" && (
          <>
            <ActionButton
              label="Approve"
              busy="Approving…"
              pending={review.isPending && event === "approve"}
              disabled={review.isPending || comment.isPending}
              onClick={() => {
                submit("approve");
              }}
            />
            <ActionButton
              label="Request changes"
              busy="Requesting…"
              pending={review.isPending && event === "changes"}
              disabled={review.isPending || comment.isPending || empty}
              onClick={() => {
                submit("changes");
              }}
            />
          </>
        )}
      </div>
    </section>
  );
}

function CommentEntry({ comment }: { comment: Comment }) {
  const author = comment.author ?? "Someone";

  return (
    <article className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
      <p className="type-caption text-muted-foreground">
        {comment.verdict
          ? `${author} ${verdicts[comment.verdict]} ${ago(comment.created)}`
          : `${author} commented ${ago(comment.created)}`}
      </p>
      <RenderedHtml html={comment.html} />
    </article>
  );
}
