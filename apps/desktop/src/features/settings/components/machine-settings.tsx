import { DotsThreeIcon, PlusIcon } from "@phosphor-icons/react";
import { Link } from "@tanstack/react-router";
import { cn } from "cn";
import { useState, useSyncExternalStore } from "react";

import { RenameMachine } from "@/features/settings/components/rename-machine";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Spinner } from "@/components/ui/spinner";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { RemoveMachine } from "@/features/settings/components/remove-machine";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { useChangeMachine } from "@/features/settings/hooks/use-change-machine";
import { useMachines } from "@/hooks/use-machines";
import type { Machine } from "@/lib/machines";

interface Props {
  onAdd: () => void;
}

export function MachineSettings({ onAdd }: Props) {
  const { all, connected } = useMachines();

  return (
    <SettingsSection
      id="machines-heading"
      title="Machines"
      action={
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Add a machine"
                nativeButton={false}
                render={<Link to="/onboarding/where" />}
                onClick={onAdd}
              />
            }
          >
            <PlusIcon />
          </TooltipTrigger>
          <TooltipContent side="top">Add a machine</TooltipContent>
        </Tooltip>
      }
    >
      {all.map((machine) => {
        const on = connected.includes(machine);

        return (
          <Row
            key={machine.id}
            machine={machine}
            connected={on}
            last={on && connected.length === 1}
          />
        );
      })}
    </SettingsSection>
  );
}

interface RowProps {
  machine: Machine;
  connected: boolean;
  last: boolean;
}

function Row({ machine, connected, last }: RowProps) {
  const [renaming, setRenaming] = useState(false);
  const [removing, setRemoving] = useState(false);
  const toggle = useChangeMachine(connected ? "disconnect" : "connect");
  const target = machine.target === machine.name ? undefined : machine.target;

  return (
    <SettingsRow
      leading={<Dot machine={machine} enabled={connected} />}
      label={
        <>
          {machine.name}
          {target && (
            <>
              <span aria-hidden className="mx-2">
                |
              </span>
              {target}
            </>
          )}
        </>
      }
    >
      {toggle.isPending && <Spinner aria-hidden />}
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={`Actions for ${machine.name}`}
            />
          }
        >
          <DotsThreeIcon />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem
            onClick={() => {
              setRenaming(true);
            }}
          >
            Rename
          </DropdownMenuItem>
          {!last && (
            <>
              <DropdownMenuItem
                disabled={toggle.isPending}
                onClick={() => {
                  toggle.mutate(machine);
                }}
              >
                {connected ? "Disconnect" : "Connect"}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                variant="destructive"
                onClick={() => {
                  setRemoving(true);
                }}
              >
                Remove
              </DropdownMenuItem>
            </>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
      <RenameMachine
        machine={machine}
        open={renaming}
        onOpenChange={setRenaming}
      />
      <RemoveMachine
        machine={machine}
        open={removing}
        onOpenChange={setRemoving}
      />
    </SettingsRow>
  );
}

const colors = {
  connected: "bg-success",
  disconnected: "bg-destructive",
  connecting: "bg-muted-foreground",
  idle: "bg-muted-foreground",
  off: "bg-muted-foreground",
};

const labels = {
  connected: "Connected",
  disconnected: "Not connected",
  connecting: "Connecting",
  idle: "Not connected",
  off: "Disconnected",
};

interface DotProps {
  machine: Machine;
  enabled: boolean;
}

function Dot({ machine, enabled }: DotProps) {
  const { state } = useSyncExternalStore(
    machine.client.subscribe,
    machine.client.getSnapshot,
  );
  const shown = enabled ? state : "off";

  return (
    <span
      role="img"
      aria-label={labels[shown]}
      className={cn("size-2 shrink-0 rounded-full", colors[shown])}
    />
  );
}
