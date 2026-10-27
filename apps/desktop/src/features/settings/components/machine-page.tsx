import { DotsThreeIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Spinner } from "@/components/ui/spinner";
import { StatusDot } from "@/features/settings/components/status-dot";
import { NotesSettings } from "@/features/settings/components/notes-settings";
import { RemoveMachine } from "@/features/settings/components/remove-machine";
import { RenameMachine } from "@/features/settings/components/rename-machine";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { ShellSettings } from "@/features/settings/components/shell-settings";
import { useChangeMachine } from "@/features/settings/hooks/use-change-machine";
import { statusLabels, useStatus } from "@/features/settings/hooks/use-status";
import { useMachines } from "@/hooks/use-machines";
import type { Machine } from "@/lib/machines";

export function MachinePage({ machine }: { machine: Machine }) {
  const { connected } = useMachines();
  const on = connected.includes(machine);
  const last = on && connected.length === 1;
  const status = useStatus(machine, on);
  const toggle = useChangeMachine(on ? "disconnect" : "connect");
  const [renaming, setRenaming] = useState(false);
  const [removing, setRemoving] = useState(false);
  const target = machine.target === machine.name ? undefined : machine.target;

  return (
    <>
      <SettingsSection id="machine-heading" title={machine.name}>
        <SettingsRow
          leading={<StatusDot status={status} />}
          label={statusLabels[status]}
          detail={target}
        >
          {toggle.isPending && <Spinner aria-hidden />}
          {!last && (
            <Button
              variant="ghost"
              size="sm"
              disabled={toggle.isPending}
              onClick={() => {
                toggle.mutate(machine);
              }}
            >
              {on ? "Disconnect" : "Connect"}
            </Button>
          )}
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
        </SettingsRow>
      </SettingsSection>
      {on && (
        <>
          <ShellSettings machine={machine} />
          <NotesSettings machine={machine} />
        </>
      )}
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
    </>
  );
}
