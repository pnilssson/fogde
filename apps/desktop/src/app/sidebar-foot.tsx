import { GearSixIcon } from "@phosphor-icons/react";

import { Layer } from "@/components/frame";
import { NavItem } from "@/components/nav-item";
import { SidebarMenu } from "@/components/ui/sidebar";
import { UpdateItem } from "@/app/update-item";
import { MonitorItem } from "@/features/monitor/components/monitor-item";
import { UsageMeters } from "@/features/usage/components/usage-meters";
import type { Machine } from "@/lib/machines";
import { openPage, usePage } from "@/lib/page";
import { keys, shortcuts } from "@/lib/shortcuts";

export function SidebarFoot({ machines }: { machines: Machine[] }) {
  const page = usePage();

  return (
    <>
      <Layer className="shrink-0 p-2 [&:not(:has(button,li))]:hidden">
        <UsageMeters machines={machines} />
      </Layer>
      <Layer className="shrink-0 p-2">
        <SidebarMenu>
          <UpdateItem />
          <MonitorItem
            machines={machines}
            active={page === "monitor"}
            onOpen={(chosen) => {
              openPage("monitor", chosen.id);
            }}
          />
          <NavItem
            icon={<GearSixIcon />}
            label="Settings"
            active={page === "settings"}
            trailing={
              <span className="type-caption">{keys(shortcuts.settings)}</span>
            }
            onClick={() => {
              openPage("settings");
            }}
          />
        </SidebarMenu>
      </Layer>
    </>
  );
}
