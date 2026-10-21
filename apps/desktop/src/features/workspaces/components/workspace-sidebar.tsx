import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  useSidebar,
} from "@/components/ui/sidebar";
import { MachineGroup } from "@/features/workspaces/components/machine-group";
import { SearchButton } from "@/features/workspaces/components/search-button";
import type { useGroups } from "@/features/workspaces/hooks/use-terminals";
import type { Selection } from "@/features/workspaces/lib/selection";

interface Props {
  groups: ReturnType<typeof useGroups>;
  selected: Selection | undefined;
  active: Selection | undefined;
  onSelect: (workspace: Selection, terminal?: Selection) => void;
  onStart: (workspace: Selection, folder: string) => void;
}

export function WorkspaceSidebar({
  groups,
  selected,
  active,
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
            active={active}
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
