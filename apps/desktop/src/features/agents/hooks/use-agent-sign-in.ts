import { useQueryClient } from "@tanstack/react-query";

import type { Agent, AgentPrompt } from "@/generated/protocol";
import { useSignInFlow } from "@/hooks/use-sign-in-flow";
import { agents, agentsQuery } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

type Prompt = Exclude<AgentPrompt, { kind: "done" }>;

export function useAgentSignIn(machine: Machine) {
  const queries = useQueryClient();
  const flow = useSignInFlow<Agent, Prompt>({
    start: async (agent) => {
      const prompt = await machine.client.call("agents.startSignIn", agent);

      return prompt.kind === "done" ? undefined : prompt;
    },
    link: (agent, prompt) =>
      prompt.kind === "browser"
        ? undefined
        : { url: prompt.url, domains: agents[agent].domains },
    waits: (prompt) => prompt.kind !== "paste",
    finish: (agent, code) =>
      machine.client.call("agents.finishSignIn", { agent, code }),
    cancel: (agent) => machine.client.call("agents.cancelSignIn", agent),
    refresh: () => queries.invalidateQueries(agentsQuery(machine)),
    failed: (agent) => `Could not connect ${agents[agent].account}`,
    stuck: (agent) => `Could not cancel signing in to ${agents[agent].account}`,
  });
  const { active } = flow;

  return {
    ...flow,
    active: active && { agent: active.key, prompt: active.prompt },
  };
}
