import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  notice: string;
  busy?: boolean;
  retry?: string;
  onRetry?: () => void;
}

export function TileStatus({ notice, busy = false, retry, onRetry }: Props) {
  return (
    <div className="flex items-center gap-2 p-2" role="status" aria-busy={busy}>
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
