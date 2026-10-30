import { ArrowSquareOutIcon } from "@phosphor-icons/react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useId } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { RenderedHtml } from "@/features/github/components/rendered-html";
import { ago } from "@/features/github/lib/scope";
import type { Thread } from "@/generated/protocol";

export function Threads({ threads }: { threads: Thread[] }) {
  const id = useId();
  if (threads.length === 0) {
    return null;
  }

  return (
    <section aria-labelledby={id} className="flex flex-col gap-2">
      <h3 id={id} className="type-title">
        {threads.length === 1
          ? "1 unresolved conversation"
          : `${String(threads.length)} unresolved conversations`}
      </h3>
      {threads.map((thread) => (
        <Conversation key={thread.comments[0]?.id} thread={thread} />
      ))}
    </section>
  );
}

function Conversation({ thread }: { thread: Thread }) {
  const url = thread.comments[0]?.url;
  const place =
    thread.line === null
      ? thread.path
      : `${thread.path}:${String(thread.line)}`;

  return (
    <article className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
      <div className="flex min-w-0 items-center gap-2">
        <span className="min-w-0 flex-1 truncate type-code">{place}</span>
        {thread.outdated && <Badge variant="outline">Outdated</Badge>}
        {url && (
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={`Open the conversation on ${place} on GitHub`}
            onClick={() => void openUrl(url)}
          >
            <ArrowSquareOutIcon />
          </Button>
        )}
      </div>
      {thread.comments.map((comment) => (
        <div key={comment.id} className="flex flex-col gap-2">
          <p className="type-caption text-muted-foreground">
            {comment.author ?? "Someone"} {ago(comment.created)}
          </p>
          <RenderedHtml html={comment.html} />
        </div>
      ))}
    </article>
  );
}
