import {
  GithubLogoIcon,
  NotebookIcon,
  SquaresFourIcon,
} from "@phosphor-icons/react";

import { Layer } from "@/components/frame";
import { NavItem } from "@/components/nav-item";
import { SidebarMenu } from "@/components/ui/sidebar";
import { showTiles } from "@/features/workspaces/lib/tiles";
import { openPage, usePage } from "@/lib/page";

export function PageNav({ machine }: { machine: string | undefined }) {
  const page = usePage();

  return (
    <Layer aria-label="Pages" className="shrink-0 p-2">
      <SidebarMenu>
        <NavItem
          icon={<SquaresFourIcon />}
          label="Workspace"
          active={page === undefined}
          onClick={showTiles}
        />
        <NavItem
          icon={<GithubLogoIcon />}
          label="GitHub"
          active={page === "github"}
          onClick={() => {
            openPage("github", machine);
          }}
        />
        <NavItem
          icon={<NotebookIcon />}
          label="Notes"
          active={page === "notes"}
          onClick={() => {
            openPage("notes", machine);
          }}
        />
      </SidebarMenu>
    </Layer>
  );
}
