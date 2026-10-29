import { cn } from "cn";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  notice: string;
  busy?: boolean;
  retry?: string;
  onRetry?: () => void;
  fill?: boolean;
}

export function TileStatus({
  notice,
  busy = false,
  retry,
  onRetry,
  fill = false,
}: Props) {
  return (
    <div
      className={cn(
        "flex items-center gap-2 p-2",
        fill && "flex-1 justify-center",
      )}
      role="status"
      aria-busy={busy}
    >
      {busy && <Spinner aria-hidden />}
      <span className="type-caption">{notice}</span>
      {retry && onRetry && (
        <Button size="sm" variant="secondary" onClick={onRetry}>
          {retry}
        </Button>
      )}
    </div>
  );
}
