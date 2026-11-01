import type { ReactNode } from "react";

import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";

interface Props {
  icon: ReactNode;
  label: string;
  active: boolean;
  trailing?: ReactNode;
  onClick: () => void;
}

export function NavItem({ icon, label, active, trailing, onClick }: Props) {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        isActive={active}
        aria-current={active ? "page" : undefined}
        onClick={onClick}
      >
        {icon}
        <span className="min-w-0 flex-1 truncate">{label}</span>
        {trailing}
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
