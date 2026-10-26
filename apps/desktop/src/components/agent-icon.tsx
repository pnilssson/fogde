import claude from "@/assets/brands/claude-spark.svg";
import copilotBlack from "@/assets/brands/copilot-black.svg";
import copilotWhite from "@/assets/brands/copilot-white.svg";
import openaiBlack from "@/assets/brands/openai-blossom-black.svg";
import openaiWhite from "@/assets/brands/openai-blossom-white.svg";
import type { Agent } from "@/generated/protocol";

const icons: Record<Agent, { light: string; dark?: string }> = {
  claude: { light: claude },
  codex: { light: openaiBlack, dark: openaiWhite },
  copilot: { light: copilotBlack, dark: copilotWhite },
};

interface Props {
  agent: Agent;
}

export function AgentIcon({ agent }: Props) {
  const { light, dark } = icons[agent];
  if (!dark) {
    return <img src={light} alt="" className="size-4 shrink-0" />;
  }

  return (
    <>
      <img src={light} alt="" className="size-4 shrink-0 dark:hidden" />
      <img src={dark} alt="" className="hidden size-4 shrink-0 dark:block" />
    </>
  );
}
