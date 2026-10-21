import { TerminalWindowIcon } from "@phosphor-icons/react";

import { AgentIcon } from "@/components/agent-icon";
import type { Agent } from "@/generated/protocol";

interface Props {
  agent: Agent | null;
}

export function TerminalIcon({ agent }: Props) {
  return agent ? (
    <AgentIcon agent={agent} />
  ) : (
    <TerminalWindowIcon className="size-4 shrink-0" />
  );
}
