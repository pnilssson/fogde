import { ArrowSquareOutIcon, PencilSimpleIcon } from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";

import { Scroll } from "@/components/frame";
import { TileStatus } from "@/components/tile-status";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { detailQuery, type Kind } from "@/features/github/api/github";
import { ActionButton } from "@/features/github/components/action-button";
import { Conversation } from "@/features/github/components/conversation";
import { PickMenu } from "@/features/github/components/pick-menu";
import { PullPanel } from "@/features/github/components/pull-panel";
import { RenderedHtml } from "@/features/github/components/rendered-html";
import { StateIcon } from "@/features/github/components/rows";
import { TextDialog } from "@/features/github/components/text-dialog";
import { useReported } from "@/features/github/hooks/use-reported";
import { useWrite } from "@/features/github/hooks/use-write";
import { ago } from "@/features/github/lib/scope";
import type { CheckoutStatus, Detail, Repo } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  kind: Kind;
  repo: Repo;
  number: number;
  checkouts: CheckoutStatus[];
}

export function ItemDetail({ machine, kind, repo, number, checkouts }: Props) {
  const detail = useQuery(detailQuery(machine, kind, repo, number));

  useReported(
    `Could not refresh #${String(number)}`,
    detail.data ? detail.error : null,
  );

  if (detail.isError && !detail.data) {
    return (
      <TileStatus
        notice={`Could not load #${String(number)}`}
        retry="Try again"
        fill
        onRetry={() => {
          void detail.refetch();
        }}
      />
    );
  }
  if (!detail.data) {
    return <TileStatus notice={`Loading #${String(number)}…`} busy fill />;
  }

  return (
    <Shown
      machine={machine}
      kind={kind}
      repo={repo}
      detail={detail.data}
      checkouts={checkouts}
    />
  );
}

interface ShownProps {
  machine: Machine;
  kind: Kind;
  repo: Repo;
  detail: Detail;
  checkouts: CheckoutStatus[];
}

function Shown({ machine, kind, repo, detail, checkouts }: ShownProps) {
  const [editing, setEditing] = useState(false);
  const edit = useWrite(machine, "Could not save the changes");
  const state = useWrite(
    machine,
    kind === "pull"
      ? "Could not change the pull request"
      : "Could not change the issue",
  );
  const pick = useWrite(machine, "Could not save the change");
  const { number, pull } = detail;
  const pullKind = kind === "pull";
  const open = detail.state === "open";
  const people =
    (method: "github.setAssignees" | "github.setReviewers") =>
    (_: string[], add: string[], remove: string[]) =>
      pick.mutateAsync((client) =>
        client.call(method, { repo, number, add, remove }),
      );

  return (
    <Scroll>
      <article className="flex flex-col gap-4">
        <header className="flex flex-col gap-2">
          <div className="flex items-start gap-2">
            <span className="flex h-lh items-center type-title">
              <StateIcon
                kind={kind}
                state={detail.state}
                draft={pull?.draft ?? false}
              />
            </span>
            <h2 className="min-w-0 flex-1 type-title">
              {detail.title}{" "}
              <span className="text-muted-foreground">#{number}</span>
            </h2>
            {detail.editable && (
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Edit"
                onClick={() => {
                  setEditing(true);
                }}
              >
                <PencilSimpleIcon />
              </Button>
            )}
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Open on GitHub"
              onClick={() => void openUrl(detail.url)}
            >
              <ArrowSquareOutIcon />
            </Button>
          </div>
          <p className="type-caption text-muted-foreground">
            {pull
              ? `${detail.author ?? "Someone"} wants to merge ${pull.head} into ${pull.base}, opened ${ago(detail.created)}`
              : `${detail.author ?? "Someone"} opened this ${ago(detail.created)}`}
            {` in ${repo.owner}/${repo.name}`}
          </p>
          <div className="flex flex-wrap items-center gap-2">
            {detail.labels.map((label) => (
              <Badge key={label.name} variant="outline">
                {label.name}
              </Badge>
            ))}
            {detail.editable && (
              <>
                <PickMenu
                  machine={machine}
                  repo={repo}
                  label="Labels"
                  source="labels"
                  chosen={detail.labels.map(({ name }) => name)}
                  onChange={(next) =>
                    pick.mutateAsync((client) =>
                      client.call("github.setLabels", {
                        repo,
                        number,
                        labels: next,
                      }),
                    )
                  }
                />
                <PickMenu
                  machine={machine}
                  repo={repo}
                  label="Assignees"
                  source="people"
                  chosen={detail.assignees}
                  onChange={people("github.setAssignees")}
                />
                {pull && (
                  <PickMenu
                    machine={machine}
                    repo={repo}
                    label="Reviewers"
                    source="people"
                    chosen={pull.reviewers}
                    onChange={people("github.setReviewers")}
                  />
                )}
              </>
            )}
            {detail.editable && detail.state !== "merged" && (
              <ActionButton
                label={open ? "Close" : "Reopen"}
                busy={open ? "Closing…" : "Reopening…"}
                pending={state.isPending}
                onClick={() => {
                  state.mutate((client) =>
                    client.call("github.setState", {
                      repo,
                      number,
                      pull: pullKind,
                      open: !open,
                    }),
                  );
                }}
              />
            )}
          </div>
          {detail.assignees.length > 0 && (
            <p className="type-caption text-muted-foreground">
              Assigned to {detail.assignees.join(", ")}
            </p>
          )}
          {pull && pull.reviewers.length > 0 && (
            <p className="type-caption text-muted-foreground">
              Waiting on {pull.reviewers.join(", ")}
            </p>
          )}
        </header>
        {pull && (
          <PullPanel
            machine={machine}
            repo={repo}
            detail={detail}
            pull={pull}
            checkouts={checkouts}
          />
        )}
        <RenderedHtml html={detail.html} empty="No description." />
        <Conversation
          machine={machine}
          kind={kind}
          repo={repo}
          detail={detail}
        />
      </article>
      <TextDialog
        open={editing}
        heading={pullKind ? "Edit pull request" : "Edit issue"}
        action="Save"
        busy="Saving…"
        initial={{ title: detail.title, body: detail.body }}
        onOpenChange={setEditing}
        onSubmit={async ({ title, body }) => {
          await edit.mutateAsync((client) =>
            client.call("github.edit", {
              repo,
              number,
              pull: pullKind,
              title,
              body,
            }),
          );
          setEditing(false);
        }}
      />
    </Scroll>
  );
}
