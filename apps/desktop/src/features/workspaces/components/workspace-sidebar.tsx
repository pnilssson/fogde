import type { ReactNode } from "react";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarMenu,
  useSidebar,
} from "@/components/ui/sidebar";
import { UpdateItem } from "@/components/update-item";
import { MachineGroup } from "@/features/workspaces/components/machine-group";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import type { Selection, View } from "@/features/workspaces/lib/selection";

interface Props {
  groups: ReturnType<typeof useGroups>;
  selected: View | undefined;
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
  meters: ReactNode;
}

export function WorkspaceSidebar({
  groups,
  selected,
  onSelect,
  onStart,
  meters,
}: Props) {
  const { setOpenMobile } = useSidebar();

  return (
    <Sidebar className="absolute h-full group-data-[side=left]:border-r-0">
      <SidebarContent className="gap-4 pt-2">
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
      <SidebarFooter className="gap-0">
        {meters}
        <SidebarMenu>
          <UpdateItem />
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
