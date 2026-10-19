import { TerminalWindowIcon } from "@phosphor-icons/react";

import {
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import type { Selection } from "@/features/terminals/lib/selection";
import { dragType, endDrag, startDrag } from "@/features/terminals/lib/tiles";
import type { Terminal } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  terminal: Terminal;
  active: boolean;
  onSelect: (selection: Selection) => void;
}

export function TerminalItem({ machine, terminal, active, onSelect }: Props) {
  const selection = { machine: machine.id, id: terminal.id };

  return (
    <SidebarMenuSubItem>
      <SidebarMenuSubButton
        isActive={active}
        render={<button type="button" />}
        draggable
        onDragStart={(event) => {
          event.dataTransfer.setData(dragType, JSON.stringify(selection));
          startDrag(selection);
        }}
        onDragEnd={endDrag}
        onClick={() => {
          onSelect(selection);
        }}
      >
        <TerminalWindowIcon weight={active ? "fill" : "regular"} />
        <span>{terminal.name}</span>
      </SidebarMenuSubButton>
    </SidebarMenuSubItem>
  );
}
