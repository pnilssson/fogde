import { DesktopIcon, HardDrivesIcon, PlusIcon } from "@phosphor-icons/react";
import { Link } from "@tanstack/react-router";

import { Button } from "@/components/ui/button";
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { StatusDot } from "@/features/settings/components/status-dot";
import { useStatus } from "@/features/settings/hooks/use-status";
import { useMachines } from "@/hooks/use-machines";
import { isComputer, type Machine } from "@/lib/machines";

interface Props {
  selected: string | undefined;
  onSelect: (machine: Machine) => void;
  onAdd: () => void;
}

export function MachineNav({ selected, onSelect, onAdd }: Props) {
  const { all, connected } = useMachines();

  return (
    <SidebarGroup className="p-0">
      <div className="flex items-center">
        <SidebarGroupLabel
          id="machines-heading"
          className="min-w-0 flex-1 type-caption text-muted-foreground"
        >
          Machines
        </SidebarGroupLabel>
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
      </div>
      <SidebarMenu aria-labelledby="machines-heading">
        {all.map((machine) => (
          <Item
            key={machine.id}
            machine={machine}
            connected={connected.includes(machine)}
            active={machine.id === selected}
            onSelect={onSelect}
          />
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}

interface ItemProps {
  machine: Machine;
  connected: boolean;
  active: boolean;
  onSelect: (machine: Machine) => void;
}

function Item({ machine, connected, active, onSelect }: ItemProps) {
  const status = useStatus(machine, connected);

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        isActive={active}
        aria-current={active ? "page" : undefined}
        onClick={() => {
          onSelect(machine);
        }}
      >
        {isComputer(machine) ? <DesktopIcon /> : <HardDrivesIcon />}
        <span className="min-w-0 flex-1 truncate">{machine.name}</span>
        <StatusDot status={status} labelled />
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
