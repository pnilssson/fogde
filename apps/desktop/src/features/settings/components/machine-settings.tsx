import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useSyncExternalStore } from "react";

import { DoneBadge } from "@/components/done-badge";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "@/components/ui/toast";
import { useMachines } from "@/hooks/use-machines";
import { callOwn } from "@/lib/host";
import { computer, forget, machinesQuery, type Machine } from "@/lib/machines";

interface Props {
  onAdd: () => void;
}

export function MachineSettings({ onAdd }: Props) {
  const machines = useMachines();
  const queries = useQueryClient();
  const remove = useMutation({
    mutationFn: (machine: Machine) =>
      machine === computer
        ? callOwn("machines.removeComputer")
        : callOwn("server.remove", machine.id),
    onSuccess: async (_, machine) => {
      forget(machine.id);
      queries.removeQueries({ queryKey: [machine.id] });
      await queries.invalidateQueries(machinesQuery);
    },
    onError: (error, machine) => {
      toast.add({
        type: "error",
        title: `Could not remove ${machine.name}`,
        description: error.message,
      });
    },
  });

  return (
    <section
      className="flex flex-col items-start gap-4"
      aria-labelledby="machines-heading"
    >
      <h2 id="machines-heading" className="type-subtitle">
        Machines
      </h2>
      <div className="flex w-full flex-col gap-2">
        {machines.map((machine) => {
          const removing = remove.isPending && remove.variables === machine;

          return (
            <div key={machine.id} className="flex h-8 items-center gap-2">
              <span className="min-w-0 flex-1 truncate type-body">
                {machine.name}
              </span>
              <Status machine={machine} />
              {machines.length > 1 && (
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={remove.isPending}
                  aria-busy={removing}
                  onClick={() => {
                    remove.mutate(machine);
                  }}
                >
                  {removing && <Spinner aria-hidden data-icon="inline-start" />}
                  {removing ? "Removing…" : "Remove"}
                </Button>
              )}
            </div>
          );
        })}
      </div>
      <Button
        variant="secondary"
        size="sm"
        nativeButton={false}
        render={<Link to="/onboarding/where" />}
        onClick={onAdd}
      >
        Add a machine
      </Button>
    </section>
  );
}

function Status({ machine }: { machine: Machine }) {
  const { state } = useSyncExternalStore(
    machine.client.subscribe,
    machine.client.getSnapshot,
  );

  if (state === "connected") {
    return <DoneBadge>Connected</DoneBadge>;
  }

  if (state === "connecting") {
    return (
      <Badge variant="secondary">
        <Spinner aria-hidden />
        Connecting
      </Badge>
    );
  }

  return <Badge variant="secondary">Not connected</Badge>;
}
