import { useNow } from "@/hooks/use-now";

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

function short(elapsed: number) {
  const minutes = Math.floor(elapsed / 60_000);
  if (minutes < 1) {
    return "now";
  }
  if (minutes < 60) {
    return `${String(minutes)}m`;
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${String(hours)}h`;
  }

  return `${String(Math.floor(hours / 24))}d`;
}
