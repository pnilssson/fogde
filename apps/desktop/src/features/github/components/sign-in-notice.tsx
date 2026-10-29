import { openUrl } from "@tauri-apps/plugin-opener";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useSignIn } from "@/features/github/hooks/use-sign-in";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  host: string;
}

export function SignInNotice({ machine, host }: Props) {
  const { prompt, starting, begin } = useSignIn(machine);

  if (prompt) {
    return (
      <div className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
        <p className="type-body">
          Enter <span className="type-code">{prompt.code}</span> on {host}.
        </p>
        <Button
          size="sm"
          variant="secondary"
          className="w-fit"
          onClick={() => void openUrl(prompt.url)}
        >
          Open {host}
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
      <p className="type-body">
        Sign in to {host} to see its pull requests and issues.
      </p>
      <Button
        size="sm"
        className="w-fit"
        disabled={starting !== undefined}
        aria-busy={starting !== undefined}
        onClick={() => {
          begin(host);
        }}
      >
        {starting !== undefined && (
          <Spinner aria-hidden data-icon="inline-start" />
        )}
        {starting !== undefined ? "Signing in…" : "Sign in"}
      </Button>
    </div>
  );
}
