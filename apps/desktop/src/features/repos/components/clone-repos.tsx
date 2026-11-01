import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { useQueries } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

import { ActionButton } from "@/components/action-button";
import { Button } from "@/components/ui/button";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { remoteReposQuery } from "@/features/repos/api/repos";
import { RepoList, type Progress } from "@/features/repos/components/repo-list";
import { repoKey } from "@/features/repos/lib/repo-key";
import type { RemoteRepo } from "@/generated/protocol";
import { report } from "@/lib/report";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  hosts: string[];
  finishing: boolean;
  onBack: () => void;
  onContinue: () => void;
}

export function CloneRepos({
  machine,
  hosts,
  finishing,
  onBack,
  onContinue,
}: Props) {
  const results = useQueries({
    queries: hosts.map((host) => ({
      ...remoteReposQuery(machine, host),
      refetchOnWindowFocus: true,
    })),
  });
  const [filter, setFilter] = useState("");
  const [cloning, setCloning] = useState<Record<string, Progress>>({});
  const reported = useRef(new Set<string>());

  useEffect(() => {
    for (const [index, host] of hosts.entries()) {
      const error = results[index]?.error;
      if (!error) {
        reported.current.delete(host);
      } else if (!reported.current.has(host)) {
        reported.current.add(host);
        report(`Could not list your repos on ${host}`, error);
      }
    }
  });

  const repos = results.flatMap((result) => result.data ?? []);
  const progress = (repo: RemoteRepo): Progress | undefined =>
    repo.cloned ? "cloned" : cloning[repoKey(repo)];

  const clone = async ({ host, owner, name }: RemoteRepo) => {
    const key = repoKey({ host, owner, name });
    setCloning((current) => ({ ...current, [key]: "cloning" }));

    try {
      await machine.client.call("repos.clone", { host, owner, name });
      setCloning((current) => ({ ...current, [key]: "cloned" }));
    } catch (error) {
      setCloning((current) =>
        Object.fromEntries(
          Object.entries(current).filter(([entry]) => entry !== key),
        ),
      );
      report(`Could not clone ${name}`, error);
    }
  };

  const search = filter.trim().toLowerCase();
  const shown = repos.filter((repo) =>
    `${repo.owner}/${repo.name}`.toLowerCase().includes(search),
  );

  return (
    <section className="flex w-sm max-w-full flex-col items-start gap-4">
      <h1 className="type-display">Clone your repos</h1>
      <p className="type-body">
        Pick the repos you work on. They are cloned into your repos folder.
      </p>
      <InputGroup>
        <InputGroupAddon>
          <MagnifyingGlassIcon />
        </InputGroupAddon>
        <InputGroupInput
          aria-label="Filter repos"
          placeholder="Filter repos"
          spellCheck={false}
          value={filter}
          onChange={(event) => {
            setFilter(event.target.value);
          }}
        />
      </InputGroup>
      <RepoList
        hosts={hosts}
        repos={shown}
        progress={progress}
        onClone={(repo) => void clone(repo)}
      />
      <div className="flex gap-2">
        <Button variant="ghost" size="sm" onClick={onBack}>
          Back
        </Button>
        <ActionButton
          label="Continue"
          busy="Finishing…"
          pending={finishing}
          onClick={onContinue}
        />
      </div>
    </section>
  );
}
