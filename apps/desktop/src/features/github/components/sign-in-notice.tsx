import { openUrl } from "@tauri-apps/plugin-opener";

import { Button } from "@/components/ui/button";
import { ActionButton } from "@/components/action-button";
import { useSignIn } from "@/features/github/hooks/use-sign-in";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  host: string;
}

export function SignInNotice({ machine, host }: Props) {
  const { prompt, starting, begin, cancel } = useSignIn(machine);

  if (prompt) {
    return (
      <div className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
        <p className="type-body">
          Enter <span className="type-code">{prompt.code}</span> on {host}.
        </p>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="secondary"
            onClick={() => void openUrl(prompt.url)}
          >
            Open {host}
          </Button>
          <Button size="sm" variant="ghost" onClick={cancel}>
            Cancel
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2 rounded-lg bg-surface-4 p-2 shadow-surface-1">
      <p className="type-body">
        Sign in to {host} to see its pull requests and issues.
      </p>
      <ActionButton
        variant="default"
        className="w-fit"
        label="Sign in"
        busy="Signing in…"
        pending={starting !== undefined}
        onClick={() => {
          begin(host);
        }}
      />
    </div>
  );
}
