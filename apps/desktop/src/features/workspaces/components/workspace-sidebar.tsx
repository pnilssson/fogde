import { NotebookIcon, PulseIcon } from "@phosphor-icons/react";
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
import { PageItem } from "@/features/workspaces/components/page-item";
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
          <PageItem
            page="notes"
            machine={selected?.machine}
            icon={<NotebookIcon />}
            label="Notes"
            onOpen={() => {
              setOpenMobile(false);
            }}
          />
          <PageItem
            page="monitor"
            machine={selected?.machine}
            icon={<PulseIcon />}
            label="Monitor"
            onOpen={() => {
              setOpenMobile(false);
            }}
          />
        </SidebarMenu>
      </SidebarHeader>
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
      <SidebarFooter>
        <SidebarMenu>
          <UpdateItem />
          <SidebarMenuItem>{settings}</SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
