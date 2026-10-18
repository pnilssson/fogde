import { useSuspenseQuery } from "@tanstack/react-query";

import { Button } from "@/components/ui/button";
import { agentsQuery } from "@/features/agents/api/agents";
import { AgentSignIn } from "@/features/agents/components/agent-sign-in";

interface Props {
  onBack: () => void;
  onContinue: () => void;
}

export function ConnectAgents({ onBack, onContinue }: Props) {
  const { data: agents } = useSuspenseQuery({
    ...agentsQuery,
    refetchOnWindowFocus: true,
  });

  return (
    <section className="flex max-w-sm flex-col items-start gap-4">
      <h1 className="type-display">Connect your agents</h1>
      <p className="type-body">
        Your coding agents run on your own plans. Sign in to at least one.
      </p>
      <div className="flex w-full flex-col gap-2">
        {agents.map(({ agent, state }) => (
          <AgentSignIn key={agent} agent={agent} state={state} />
        ))}
      </div>
      <div className="flex gap-2">
        <Button variant="ghost" size="sm" onClick={onBack}>
          Back
        </Button>
        <Button
          variant="secondary"
          size="sm"
          disabled={!agents.some(({ state }) => state === "signedIn")}
          onClick={onContinue}
        >
          Continue
        </Button>
      </div>
    </section>
  );
}
