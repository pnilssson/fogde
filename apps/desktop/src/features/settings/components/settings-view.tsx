import {
  PaletteIcon,
  SidebarSimpleIcon,
  SlidersHorizontalIcon,
} from "@phosphor-icons/react";
import { type ReactNode, useState } from "react";

import { Layer, SideScroll } from "@/components/frame";
import { PageShell } from "@/components/shell";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { AccessibilitySettings } from "@/features/settings/components/accessibility-settings";
import { AppearanceSettings } from "@/features/settings/components/appearance-settings";
import { MachineNav } from "@/features/settings/components/machine-nav";
import { MachinePage } from "@/features/settings/components/machine-page";
import { MonitorSettings } from "@/features/settings/components/monitor-settings";
import { NotificationSettings } from "@/features/settings/components/notification-settings";
import { StatsSettings } from "@/features/settings/components/stats-settings";
import { UsageSettings } from "@/features/settings/components/usage-settings";
import { useMachines } from "@/hooks/use-machines";
import { closePage } from "@/lib/page";

type Section = "general" | "appearance" | "sidebar" | { machine: string };

const sections = [
  {
    section: "general",
    label: "General",
    icon: SlidersHorizontalIcon,
    content: (
      <>
        <NotificationSettings />
        <AccessibilitySettings />
      </>
    ),
  },
  {
    section: "appearance",
    label: "Appearance",
    icon: PaletteIcon,
    content: <AppearanceSettings />,
  },
  {
    section: "sidebar",
    label: "Sidebar",
    icon: SidebarSimpleIcon,
    content: (
      <>
        <StatsSettings />
        <UsageSettings />
        <MonitorSettings />
      </>
    ),
  },
] as const;

export function SettingsView() {
  const [section, setSection] = useState<Section>("general");
  const { all } = useMachines();
  const machine =
    typeof section === "object"
      ? all.find(({ id }) => id === section.machine)
      : undefined;
  const shown = typeof section === "object" && !machine ? "general" : section;
  const entry = sections.find((each) => each.section === shown);

  let content: ReactNode;
  if (machine) {
    content = <MachinePage key={machine.id} machine={machine} />;
  } else {
    content = entry?.content;
  }

  return (
    <PageShell
      side={
        <Layer className="flex-1">
          <SideScroll>
            <nav className="flex flex-col gap-4 pt-2">
              <SidebarMenu>
                {sections.map((each) => {
                  const active = each.section === shown;

                  return (
                    <SidebarMenuItem key={each.section}>
                      <SidebarMenuButton
                        isActive={active}
                        aria-current={active ? "page" : undefined}
                        onClick={() => {
                          setSection(each.section);
                        }}
                      >
                        <each.icon />
                        <span>{each.label}</span>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  );
                })}
              </SidebarMenu>
              <MachineNav
                selected={machine?.id}
                onSelect={({ id }) => {
                  setSection({ machine: id });
                }}
                onAdd={closePage}
              />
            </nav>
          </SideScroll>
        </Layer>
      }
    >
      <ScrollArea className="min-h-0 flex-1">
        <div className="flex flex-col gap-4 p-4">{content}</div>
      </ScrollArea>
    </PageShell>
  );
}
