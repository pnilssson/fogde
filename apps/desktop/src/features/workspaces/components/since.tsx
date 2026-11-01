import { useNow } from "@/hooks/use-now";
import { short } from "@/lib/time";

export function Since({ at }: { at: number }) {
  const now = useNow();
  const time = at * 1000;

  return (
    <time
      dateTime={new Date(time).toISOString()}
      className="shrink-0 type-caption text-muted-foreground tabular-nums"
    >
      {short(now - time)}
    </time>
  );
}
