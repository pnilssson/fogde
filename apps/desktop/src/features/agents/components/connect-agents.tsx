import { useSuspenseQuery } from "@tanstack/react-query";
import { AnimatePresence } from "motion/react";
import * as m from "motion/react-m";

import { ContinueButton } from "@/components/continue-button";
import { DeviceCode } from "@/components/device-code";
import { Button } from "@/components/ui/button";
import { agentsQuery } from "@/features/agents/api/agents";
import { AgentSignIn } from "@/features/agents/components/agent-sign-in";
import { PasteCode } from "@/features/agents/components/paste-code";
import { useAgentSignIn } from "@/features/agents/hooks/use-agent-sign-in";
import { accountName } from "@/features/agents/lib/account-name";
import { swap } from "@/lib/motion";

interface Props {
  onBack: () => void;
  onContinue: () => Promise<void>;
}

export function ConnectAgents({ onBack, onContinue }: Props) {
  const { data: agents } = useSuspenseQuery({
    ...agentsQuery,
    refetchOnWindowFocus: true,
  });
  const { active, submitting, begin, paste, cancel } = useAgentSignIn();
  const prompt = active?.prompt;

  return (
    <AnimatePresence mode="wait" initial={false}>
      {active && prompt?.kind === "code" ? (
        <m.div key="code" {...swap}>
          <DeviceCode
            code={prompt.code}
            url={prompt.url}
            site={accountName[active.agent]}
            hint={`Device code sign-in needs to be on in your ${accountName[active.agent]} security settings.`}
            onCancel={cancel}
          />
        </m.div>
      ) : active && prompt?.kind === "paste" ? (
        <m.div key="paste" {...swap}>
          <PasteCode
            name={accountName[active.agent]}
            url={prompt.url}
            submitting={submitting}
            onSubmit={paste}
            onCancel={cancel}
          />
        </m.div>
      ) : (
        <m.section
          key="agents"
          {...swap}
          className="flex max-w-sm flex-col items-start gap-4"
        >
          <h1 className="type-display">Connect your agents</h1>
          <p className="type-body">
            Your coding agents run on your own plans. Sign in to at least one.
          </p>
          <div className="flex w-full flex-col gap-2">
            {agents.map(({ agent, state }) => (
              <AgentSignIn
                key={agent}
                agent={agent}
                state={state}
                signingIn={active?.agent === agent}
                onSignIn={() => {
                  if (!active) {
                    begin(agent);
                  }
                }}
                onCancel={
                  active?.agent === agent &&
                  (prompt === undefined || prompt.kind === "browser")
                    ? cancel
                    : undefined
                }
              />
            ))}
          </div>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={onBack}>
              Back
            </Button>
            <ContinueButton
              disabled={!agents.some(({ state }) => state === "signedIn")}
              onContinue={onContinue}
            />
          </div>
        </m.section>
      )}
    </AnimatePresence>
  );
}
