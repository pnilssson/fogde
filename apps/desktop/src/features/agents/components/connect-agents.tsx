import { useSuspenseQuery } from "@tanstack/react-query";

import { OnboardingProgress } from "@/components/onboarding-progress";
import { Button } from "@/components/ui/button";
import { signedInQuery } from "@/features/agents/api/agents";
import { AgentSignIn } from "@/features/agents/components/agent-sign-in";
import type { Agent } from "@/generated/protocol";

const agents: Agent[] = ["claude", "codex"];

interface Props {
  onBack: () => void;
  onContinue: () => void;
}

export function ConnectAgents({ onBack, onContinue }: Props) {
  const { data: signedIn } = useSuspenseQuery({
    ...signedInQuery,
    refetchOnWindowFocus: true,
  });

  return (
    <section className="flex max-w-sm flex-col items-start gap-4">
      <OnboardingProgress step="agents" />
      <h1 className="type-display">Connect your agents</h1>
      <p className="type-body">
        Fogde runs your coding agents on your own plans. Sign in to at least
        one.
      </p>
      <div className="flex w-full flex-col gap-2">
        {agents.map((agent) => (
          <AgentSignIn
            key={agent}
            agent={agent}
            signedIn={signedIn.includes(agent)}
          />
        ))}
      </div>
      <div className="flex gap-2">
        <Button variant="ghost" onClick={onBack}>
          Back
        </Button>
        <Button
          variant="secondary"
          disabled={signedIn.length === 0}
          onClick={onContinue}
        >
          Continue
        </Button>
      </div>
    </section>
  );
}
