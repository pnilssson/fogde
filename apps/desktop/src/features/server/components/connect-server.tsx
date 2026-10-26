import { InfoIcon } from "@phosphor-icons/react";
import { useForm } from "@tanstack/react-form";
import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { useState } from "react";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Field, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { sshHostsQuery } from "@/features/server/api/server";
import { KeyHelp } from "@/features/server/components/key-help";
import { TrustServer } from "@/features/server/components/trust-server";
import type { ServerCheck } from "@/generated/protocol";
import { callOwn } from "@/lib/host";
import { machinesQuery } from "@/lib/machines";

type Phase = "connecting" | "settingUp";

const labels: Record<Phase, string> = {
  connecting: "Connecting…",
  settingUp: "Setting up…",
};

type Problem = Exclude<ServerCheck, { state: "ready" }> & { target: string };

interface HelpProps {
  problem: Problem;
  pending: string | undefined;
  onConnect: (trust: string | null) => void;
}

function ProblemHelp({ problem, pending, onConnect }: HelpProps) {
  switch (problem.state) {
    case "untrusted":
      return (
        <TrustServer
          target={problem.target}
          fingerprint={problem.fingerprint}
          pending={pending}
          onTrust={() => {
            onConnect(problem.fingerprint);
          }}
        />
      );
    case "keyRejected":
      return (
        <KeyHelp
          target={problem.target}
          keys={problem.keys}
          pending={pending}
          onRetry={() => {
            onConnect(null);
          }}
        />
      );
  }
}

interface Props {
  onBack: () => void;
  onContinue: (machine: string) => Promise<void>;
}

export function ConnectServer({ onBack, onContinue }: Props) {
  const { data: hosts } = useSuspenseQuery(sshHostsQuery);
  const queries = useQueryClient();
  const [phase, setPhase] = useState<Phase>();
  const [problem, setProblem] = useState<Problem>();
  const shown = problem ? undefined : phase;

  const connect = async (target: string, trust: string | null) => {
    setPhase("connecting");
    try {
      const check = await callOwn("server.check", { target, trust });
      if (check.state !== "ready") {
        setProblem({ ...check, target });
        setPhase(undefined);
        return;
      }

      setProblem(undefined);
      setPhase("settingUp");
      const machine = await callOwn("server.setUp", target);
      await queries.invalidateQueries(machinesQuery);
      await onContinue(machine);
    } catch (error) {
      setPhase(undefined);
      toast.add({
        type: "error",
        title: `Could not connect to ${target}`,
        description: error instanceof Error ? error.message : String(error),
      });
    }
  };

  const form = useForm({
    defaultValues: { target: "" },
    onSubmit: ({ value }) => {
      setProblem(undefined);
      return connect(value.target.trim(), null);
    },
  });

  return (
    <section className="flex max-w-sm flex-col items-start gap-4">
      <h1 className="type-display">Connect your server</h1>
      <p className="type-body">
        Sign in with your own SSH key. The server is set up for you.
      </p>
      <form
        className="flex w-full flex-col gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          event.stopPropagation();
          void form.handleSubmit();
        }}
      >
        <form.Field
          name="target"
          validators={{
            onSubmit: z.string().trim().min(1, "Enter a server"),
          }}
        >
          {(field) => (
            <Field data-invalid={!field.state.meta.isValid}>
              <div className="flex items-center gap-1">
                <FieldLabel htmlFor={field.name}>Server</FieldLabel>
                <Tooltip>
                  <TooltipTrigger
                    render={
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon-xs"
                        aria-label="What to enter"
                      />
                    }
                  >
                    <InfoIcon />
                  </TooltipTrigger>
                  <TooltipContent>
                    user@address, or a host from your SSH config
                  </TooltipContent>
                </Tooltip>
              </div>
              <Input
                id={field.name}
                name={field.name}
                value={field.state.value}
                placeholder="root@203.0.113.7"
                aria-invalid={!field.state.meta.isValid}
                autoCapitalize="off"
                spellCheck={false}
                onBlur={field.handleBlur}
                onChange={(event) => {
                  field.handleChange(event.target.value);
                  setProblem(undefined);
                }}
              />
              {hosts.length > 0 && (
                <div className="flex flex-wrap gap-2">
                  {hosts.map((host) => (
                    <Button
                      key={host}
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        field.handleChange(host);
                        setProblem(undefined);
                      }}
                    >
                      {host}
                    </Button>
                  ))}
                </div>
              )}
              <FieldError errors={field.state.meta.errors} />
            </Field>
          )}
        </form.Field>
        {problem && (
          <ProblemHelp
            problem={problem}
            pending={phase && labels[phase]}
            onConnect={(trust) => void connect(problem.target, trust)}
          />
        )}
        <div className="flex gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={onBack}>
            Back
          </Button>
          <Button
            type="submit"
            variant="secondary"
            size="sm"
            aria-busy={shown !== undefined}
            disabled={phase !== undefined}
          >
            {shown && <Spinner aria-hidden data-icon="inline-start" />}
            {shown ? labels[shown] : "Continue"}
          </Button>
        </div>
      </form>
    </section>
  );
}
