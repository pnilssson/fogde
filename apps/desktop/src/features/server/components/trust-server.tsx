import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  target: string;
  fingerprint: string;
  pending: string | undefined;
  onTrust: () => void;
}

export function TrustServer({ target, fingerprint, pending, onTrust }: Props) {
  return (
    <Alert>
      <AlertTitle>First connection to {target}</AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        <p className="break-all">Its fingerprint is {fingerprint}.</p>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          aria-busy={pending !== undefined}
          disabled={pending !== undefined}
          onClick={onTrust}
        >
          {pending && <Spinner aria-hidden data-icon="inline-start" />}
          {pending ?? "Trust this server"}
        </Button>
      </AlertDescription>
    </Alert>
  );
}
