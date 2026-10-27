import type { ReactNode } from "react";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { UpdateItem } from "@/components/update-item";
import { MachineGroup } from "@/features/workspaces/components/machine-group";
import { SearchItem } from "@/features/workspaces/components/search-item";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import type { Selection, View } from "@/features/workspaces/lib/selection";

interface Props {
  groups: ReturnType<typeof useGroups>;
  selected: View | undefined;
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
  settings: ReactNode;
}

export function WorkspaceSidebar({
  groups,
  selected,
  onSelect,
  onStart,
  settings,
}: Props) {
  const { setOpenMobile } = useSidebar();

  return (
    <Sidebar className="absolute h-full">
      <SidebarHeader>
        <SidebarMenu>
          <SearchItem
            onOpen={() => {
              setOpenMobile(false);
            }}
          />
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        {groups.map(({ machine, workspaces }) => (
          <MachineGroup
            key={machine.id}
            machine={machine}
            workspaces={workspaces}
            selected={selected}
            onSelect={(workspace, terminal) => {
              onSelect(workspace, terminal);
              setOpenMobile(false);
            }}
            onStart={(workspace, folder) => {
              onStart(workspace, folder);
              setOpenMobile(false);
            }}
          />
        ))}
      </SidebarContent>
      <SidebarFooter>
        <SidebarMenu>
          <UpdateItem />
          <SidebarMenuItem>{settings}</SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
