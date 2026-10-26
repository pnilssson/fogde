import { useSuspenseQuery } from "@tanstack/react-query";
import { AnimatePresence } from "motion/react";
import * as m from "motion/react-m";
import type { ReactNode } from "react";

import { ContinueButton } from "@/components/continue-button";
import { DeviceCode } from "@/components/device-code";
import { Button } from "@/components/ui/button";
import { AgentSignIn } from "@/features/agents/components/agent-sign-in";
import { PasteCode } from "@/features/agents/components/paste-code";
import { useAgentSignIn } from "@/features/agents/hooks/use-agent-sign-in";
import type { Agent, AgentPrompt } from "@/generated/protocol";
import { agents, agentsQuery } from "@/lib/agents";
import type { Machine } from "@/lib/machines";
import { swap } from "@/lib/motion";

interface Props {
  machine: Machine;
  onBack: () => void;
  onContinue: () => Promise<void>;
}

interface Step {
  agent: Agent;
  prompt: Exclude<AgentPrompt, { kind: "done" }>;
  submitting: boolean;
  paste: (code: string) => void;
  cancel: () => void;
}

function step({ agent, prompt, submitting, paste, cancel }: Step): ReactNode {
  const { account } = agents[agent];

  switch (prompt.kind) {
    case "code":
      return (
        <DeviceCode
          code={prompt.code}
          url={prompt.url}
          site={account}
          hint={`Device code sign-in needs to be on in your ${account} security settings.`}
          onCancel={cancel}
        />
      );
    case "paste":
      return (
        <PasteCode
          name={account}
          url={prompt.url}
          submitting={submitting}
          onSubmit={paste}
          onCancel={cancel}
        />
      );
    case "browser":
      return undefined;
  }
}

export function ConnectAgents({ machine, onBack, onContinue }: Props) {
  const { data: statuses } = useSuspenseQuery({
    ...agentsQuery(machine),
    refetchOnWindowFocus: true,
  });
  const { active, submitting, begin, paste, cancel } = useAgentSignIn(machine);
  const prompt = active?.prompt;
  const shown =
    active &&
    prompt &&
    step({ agent: active.agent, prompt, submitting, paste, cancel });

  return (
    <AnimatePresence mode="wait" initial={false}>
      {shown ? (
        <m.div key={prompt.kind} {...swap}>
          {shown}
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
            {statuses.map(({ agent, state }) => (
              <AgentSignIn
                key={agent}
                machine={machine}
                agent={agent}
                state={state}
                signingIn={active?.agent === agent}
                onSignIn={() => {
                  if (!active) {
                    begin(agent);
                  }
                }}
                onCancel={
                  active?.agent === agent && !shown ? cancel : undefined
                }
              />
            ))}
          </div>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={onBack}>
              Back
            </Button>
            <ContinueButton
              disabled={!statuses.some(({ state }) => state === "signedIn")}
              onContinue={onContinue}
            />
          </div>
        </m.section>
      )}
    </AnimatePresence>
  );
}
