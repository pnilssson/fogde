import { MagnifyingGlassIcon } from "@phosphor-icons/react";

import { Keys } from "@/components/keys";
import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { setPaletteOpen } from "@/features/workspaces/lib/palette";
import { shortcuts } from "@/lib/shortcuts";

export function SearchItem({ onOpen }: { onOpen: () => void }) {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        className="bg-surface-2 shadow-surface-1 hover:bg-hover"
        onClick={() => {
          onOpen();
          setPaletteOpen(true);
        }}
      >
        <MagnifyingGlassIcon />
        <span className="min-w-0 flex-1 truncate">Search</span>
        <Keys letter={shortcuts.palette} className="bg-transparent px-0" />
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
