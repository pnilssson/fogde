import { GithubLogoIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { DeviceCode } from "@/features/github/components/device-code";
import { useSignIn } from "@/features/github/hooks/use-sign-in";

export function SignIn() {
  const { prompt, starting, error, begin } = useSignIn();

  if (error) {
    return (
      <section className="flex max-w-sm flex-col items-start gap-4">
        <h1 className="type-display">Sign-in didn't finish</h1>
        <p className="type-body">{error.message}</p>
        <Button onClick={begin}>Try again</Button>
      </section>
    );
  }

  if (prompt) {
    return <DeviceCode code={prompt.code} url={prompt.url} />;
  }

  return (
    <section className="flex max-w-sm flex-col items-start gap-4">
      <h1 className="type-display">Welcome to Fogde</h1>
      <p className="type-body">
        Run your coding agents like a team, not a pile of terminals.
      </p>
      <Button onClick={begin} disabled={starting}>
        {starting ? <Spinner aria-hidden /> : <GithubLogoIcon />}
        Sign in with GitHub
      </Button>
    </section>
  );
}
