import type { ReactNode } from "react";

import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { openPage, type Page, usePage } from "@/lib/page";

interface Props {
  page: Page;
  machine: string | undefined;
  icon: ReactNode;
  label: string;
  onOpen: () => void;
}

export function PageItem({ page, machine, icon, label, onOpen }: Props) {
  const active = usePage() === page;

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        isActive={active}
        aria-current={active ? "page" : undefined}
        onClick={() => {
          onOpen();
          if (!active) {
            openPage(page, machine);
          }
        }}
      >
        {icon}
        <span className="min-w-0 flex-1 truncate">{label}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
