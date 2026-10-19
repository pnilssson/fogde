import { PlusCircleIcon } from "@phosphor-icons/react";

import {
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";

interface Props {
  active: boolean;
  onSelect: () => void;
}

export function StartItem({ active, onSelect }: Props) {
  return (
    <SidebarMenuSubItem>
      <SidebarMenuSubButton
        isActive={active}
        render={<button type="button" />}
        onClick={onSelect}
      >
        <PlusCircleIcon weight={active ? "fill" : "regular"} />
        <span>New tile</span>
      </SidebarMenuSubButton>
    </SidebarMenuSubItem>
  );
}
