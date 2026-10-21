import { queryOptions } from "@tanstack/react-query";

import type { Agent } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

export const agentsQuery = (machine: Machine) =>
  queryOptions({
    queryKey: [machine.id, "agents.list"],
    queryFn: () => machine.client.call("agents.list"),
  });

export const agentName: Record<Agent, string> = {
  claude: "Claude Code",
  codex: "Codex",
  copilot: "GitHub Copilot",
};

export const signInDomains: Record<Agent, readonly string[]> = {
  claude: ["claude.com", "claude.ai", "anthropic.com"],
  codex: ["openai.com", "chatgpt.com"],
  copilot: ["github.com", "ghe.com"],
};
