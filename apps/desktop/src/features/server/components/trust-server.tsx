import { ActionButton } from "@/components/action-button";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

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
        <ActionButton
          type="button"
          label="Trust this server"
          busy={pending}
          pending={pending !== undefined}
          onClick={onTrust}
        />
      </AlertDescription>
    </Alert>
  );
}
