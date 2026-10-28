import {
  GearSixIcon,
  PaletteIcon,
  SidebarSimpleIcon,
  SlidersHorizontalIcon,
} from "@phosphor-icons/react";
import { type ReactNode, useState } from "react";

import { Keys } from "@/components/keys";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { AccessibilitySettings } from "@/features/settings/components/accessibility-settings";
import { AppearanceSettings } from "@/features/settings/components/appearance-settings";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { MachineNav } from "@/features/settings/components/machine-nav";
import { MachinePage } from "@/features/settings/components/machine-page";
import { MonitorSettings } from "@/features/settings/components/monitor-settings";
import { NotificationSettings } from "@/features/settings/components/notification-settings";
import { StatsSettings } from "@/features/settings/components/stats-settings";
import { UsageSettings } from "@/features/settings/components/usage-settings";
import { useMachines } from "@/hooks/use-machines";
import { setSettingsOpen, useSettingsOpen } from "@/lib/settings-open";
import { shortcuts } from "@/lib/shortcuts";

type Page = "general" | "appearance" | "sidebar" | { machine: string };

const pages = [
  {
    page: "general",
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
    page: "appearance",
    label: "Appearance",
    icon: PaletteIcon,
    content: <AppearanceSettings />,
  },
  {
    page: "sidebar",
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

export function SettingsDialog() {
  const open = useSettingsOpen();
  const [page, setPage] = useState<Page>("general");
  const { all } = useMachines();
  const machine =
    typeof page === "object"
      ? all.find(({ id }) => id === page.machine)
      : undefined;
  const shown = typeof page === "object" && !machine ? "general" : page;

  let content: ReactNode;
  if (machine) {
    content = <MachinePage key={machine.id} machine={machine} />;
  } else {
    content = pages.find((entry) => entry.page === shown)?.content;
  }

  return (
    <Dialog open={open} onOpenChange={setSettingsOpen}>
      <Tooltip>
        <TooltipTrigger
          render={
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Settings"
              onClick={() => {
                setSettingsOpen(true);
              }}
            />
          }
        >
          <GearSixIcon />
        </TooltipTrigger>
        <TooltipContent side="bottom">
          Settings
          <Keys letter={shortcuts.settings} />
        </TooltipContent>
      </Tooltip>
      <DialogContent className="flex h-144 max-h-[calc(100%-2rem)] gap-0 overflow-hidden p-0 sm:max-w-3xl">
        <ScrollArea className="w-52 shrink-0 border-r">
          <nav aria-label="Settings" className="flex flex-col gap-4 p-2">
            <DialogTitle className="flex h-8 items-center px-2 type-title">
              Settings
            </DialogTitle>
            <SidebarMenu>
              {pages.map((entry) => {
                const active = entry.page === shown;

                return (
                  <SidebarMenuItem key={entry.page}>
                    <SidebarMenuButton
                      isActive={active}
                      aria-current={active ? "page" : undefined}
                      onClick={() => {
                        setPage(entry.page);
                      }}
                    >
                      <entry.icon />
                      <span>{entry.label}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
            <MachineNav
              selected={machine?.id}
              onSelect={({ id }) => {
                setPage({ machine: id });
              }}
              onAdd={() => {
                setSettingsOpen(false);
              }}
            />
          </nav>
        </ScrollArea>
        <ScrollArea className="min-h-0 min-w-0 flex-1">
          <div className="flex flex-col gap-6 p-6">{content}</div>
        </ScrollArea>
      </DialogContent>
    </Dialog>
  );
}
