import claude from "@/assets/brands/claude-spark.svg";
import copilotBlack from "@/assets/brands/copilot-black.svg";
import copilotWhite from "@/assets/brands/copilot-white.svg";
import openaiBlack from "@/assets/brands/openai-blossom-black.svg";
import openaiWhite from "@/assets/brands/openai-blossom-white.svg";
import type { Agent } from "@/generated/protocol";

interface Props {
  agent: Agent;
}

export function AgentIcon({ agent }: Props) {
  switch (agent) {
    case "claude":
      return <img src={claude} alt="" className="size-4 shrink-0" />;
    case "codex":
      return <Themed light={openaiBlack} dark={openaiWhite} />;
    case "copilot":
      return <Themed light={copilotBlack} dark={copilotWhite} />;
  }
}

interface ThemedProps {
  light: string;
  dark: string;
}

function Themed({ light, dark }: ThemedProps) {
  return (
    <>
      <img src={light} alt="" className="size-4 shrink-0 dark:hidden" />
      <img src={dark} alt="" className="hidden size-4 shrink-0 dark:block" />
    </>
  );
}
