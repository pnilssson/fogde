import { useSuspenseQuery } from "@tanstack/react-query";
import { useState } from "react";

import { Connection } from "@/components/connection";
import { OnboardingProgress } from "@/components/onboarding-progress";
import { Button } from "@/components/ui/button";
import { accountsQuery } from "@/features/github/api/accounts";
import { DeviceCode } from "@/features/github/components/device-code";
import { EnterpriseForm } from "@/features/github/components/enterprise-form";
import { useSignIn } from "@/features/github/hooks/use-sign-in";

const github = "github.com";

interface Props {
  onContinue: () => void;
}

export function ConnectGitHub({ onContinue }: Props) {
  const { data: accounts } = useSuspenseQuery({
    ...accountsQuery,
    refetchOnWindowFocus: true,
  });
  const { prompt, starting, error, begin } = useSignIn();
  const [enterprise, setEnterprise] = useState(false);

  if (prompt) {
    return <DeviceCode code={prompt.code} url={prompt.url} />;
  }

  const hasGitHub = accounts.some((account) => account.host === github);
  const hasEnterprise = accounts.some((account) => account.host !== github);
  const startingEnterprise = starting !== undefined && starting !== github;

  return (
    <section className="flex max-w-sm flex-col items-start gap-4">
      <OnboardingProgress step="github" />
      <h1 className="type-display">Set up GitHub</h1>
      <p className="type-body">
        Fogde and your agents use the GitHub CLI to clone, push and open pull
        requests.
      </p>
      <div className="flex w-full flex-col gap-2">
        <Connection
          name="GitHub"
          status={
            hasGitHub
              ? "connected"
              : starting === github
                ? "connecting"
                : "disconnected"
          }
          onConnect={() => {
            if (!starting) {
              begin(github);
            }
          }}
        />
        <Connection
          name="GitHub Enterprise"
          status={
            hasEnterprise
              ? "connected"
              : startingEnterprise
                ? "connecting"
                : "disconnected"
          }
          onConnect={() => {
            setEnterprise(true);
          }}
        />
        {enterprise && !hasEnterprise && (
          <EnterpriseForm starting={startingEnterprise} onSubmit={begin} />
        )}
      </div>
      {error && <p className="type-body">{error.message}</p>}
      <Button
        variant="secondary"
        disabled={accounts.length === 0}
        onClick={onContinue}
      >
        Continue
      </Button>
    </section>
  );
}
