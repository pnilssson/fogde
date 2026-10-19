import { useQueryClient } from "@tanstack/react-query";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useRef, useState } from "react";

import { toast } from "@/components/ui/toast";
import { accountName } from "@/features/agents/lib/account-name";
import type { Agent, AgentPrompt } from "@/generated/protocol";
import { agentsQuery } from "@/lib/agents";
import type { Machine } from "@/lib/machines";

interface Active {
  agent: Agent;
  prompt?: Exclude<AgentPrompt, { kind: "done" }>;
  submitting: boolean;
}

export function useAgentSignIn(machine: Machine) {
  const queries = useQueryClient();
  const [active, setActive] = useState<Active>();
  const attempt = useRef(0);

  const current = (id: number) => attempt.current === id;
  const end = (id: number) => {
    if (current(id)) {
      attempt.current += 1;
      setActive(undefined);
    }
  };
  const done = async (id: number) => {
    if (current(id)) {
      await queries.invalidateQueries(agentsQuery(machine));
    }

    end(id);
  };
  const fail = (id: number, agent: Agent, error: unknown) => {
    if (current(id)) {
      toast.add({
        type: "error",
        title: `Could not connect ${accountName[agent]}`,
        description: error instanceof Error ? error.message : String(error),
      });
    }

    end(id);
  };
  const finish = (id: number, agent: Agent, code: string | null) => {
    setActive((shown) => shown && { ...shown, submitting: true });
    machine.client.call("agents.finishSignIn", { agent, code }).then(
      () => done(id),
      (error: unknown) => {
        fail(id, agent, error);
      },
    );
  };

  const begin = async (agent: Agent) => {
    attempt.current += 1;
    const id = attempt.current;
    setActive({ agent, submitting: false });

    let prompt: AgentPrompt;
    try {
      prompt = await machine.client.call("agents.startSignIn", agent);
    } catch (error) {
      fail(id, agent, error);
      return;
    }

    if (!current(id)) {
      return;
    }
    if (prompt.kind === "done") {
      await done(id);
      return;
    }

    setActive({ agent, prompt, submitting: false });
    if (prompt.kind !== "browser") {
      void openUrl(prompt.url);
    }
    if (prompt.kind !== "paste") {
      finish(id, agent, null);
    }
  };

  return {
    active,
    submitting: active?.submitting ?? false,
    begin: (agent: Agent) => {
      void begin(agent);
    },
    paste: (code: string) => {
      if (active) {
        finish(attempt.current, active.agent, code);
      }
    },
    cancel: () => {
      if (!active) {
        return;
      }

      end(attempt.current);
      machine.client
        .call("agents.cancelSignIn", active.agent)
        .catch((error: unknown) => {
          toast.add({
            type: "error",
            title: `Could not cancel signing in to ${accountName[active.agent]}`,
            description: error instanceof Error ? error.message : String(error),
          });
        })
        .finally(() => void queries.invalidateQueries(agentsQuery(machine)));
    },
  };
}
