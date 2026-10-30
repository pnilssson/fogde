import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import type { SshKey } from "@/generated/protocol";

interface Props {
  target: string;
  keys: SshKey[];
  pending: string | undefined;
  onRetry: () => void;
}

export function KeyHelp({ target, keys, pending, onRetry }: Props) {
  return (
    <Alert>
      <AlertTitle>
        {keys.length === 0
          ? "This computer has no SSH key to sign in with"
          : `${target} didn't accept your SSH key`}
      </AlertTitle>
      <AlertDescription className="flex flex-col items-start gap-2">
        {keys.length === 0 ? (
          <p>
            Sign-in works with the same keys as ssh on this computer: key files
            in your .ssh folder, or keys from an SSH agent. If your key lives in
            a password manager, turn on its SSH agent.
          </p>
        ) : (
          <>
            <p>ssh offered:</p>
            <ul className="flex flex-col gap-2">
              {keys.map((key) => (
                <li key={key.fingerprint} className="break-all">
                  {key.name} ({key.fingerprint})
                </li>
              ))}
            </ul>
          </>
        )}
        <p>
          The server needs the key's public half. Most providers add it when you
          create or rebuild a server.
        </p>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          aria-busy={pending !== undefined}
          disabled={pending !== undefined}
          onClick={onRetry}
        >
          {pending && <Spinner aria-hidden data-icon="inline-start" />}
          {pending ?? "Try again"}
        </Button>
      </AlertDescription>
    </Alert>
  );
}
