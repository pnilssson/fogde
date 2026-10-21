import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  useSidebar,
} from "@/components/ui/sidebar";
import { MachineGroup } from "@/features/workspaces/components/machine-group";
import { SearchButton } from "@/features/workspaces/components/search-button";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import type { Selection, View } from "@/features/workspaces/lib/selection";

interface Props {
  groups: ReturnType<typeof useGroups>;
  selected: View | undefined;
  onSelect: (view: View, terminal?: Selection) => void;
  onStart: (view: View, folder: string) => void;
}

export function WorkspaceSidebar({
  groups,
  selected,
  onSelect,
  onStart,
}: Props) {
  const { setOpenMobile } = useSidebar();

  return (
    <Sidebar className="absolute h-full">
      <SidebarHeader>
        <SearchButton
          onOpen={() => {
            setOpenMobile(false);
          }}
        />
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
    </Sidebar>
  );
}
