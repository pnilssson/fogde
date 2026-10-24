import { GearSixIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { ThemeSwitcher } from "@/components/theme-switcher";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { AccessibilitySettings } from "@/features/settings/components/accessibility-settings";
import { MachineSettings } from "@/features/settings/components/machine-settings";
import { NotificationSettings } from "@/features/settings/components/notification-settings";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";
import { ShellSettings } from "@/features/settings/components/shell-settings";
import { StatsSettings } from "@/features/settings/components/stats-settings";
import { TranslucentSwitch } from "@/features/settings/components/translucent-switch";
import { UsageSettings } from "@/features/settings/components/usage-settings";
import { frame } from "@/lib/frame";

export function SettingsDialog() {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Tooltip>
        <TooltipTrigger
          render={
            <DialogTrigger
              render={
                <Button variant="ghost" size="sm" aria-label="Settings" />
              }
            />
          }
        >
          <GearSixIcon />
        </TooltipTrigger>
        <TooltipContent side="top">Settings</TooltipContent>
      </Tooltip>
      <DialogContent className="flex max-h-[calc(100%-2rem)] flex-col">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
        </DialogHeader>
        <ScrollArea className="-mx-6 flex min-h-0 flex-col">
          <div className="flex flex-col gap-6 px-6">
            <SettingsSection id="appearance-heading" title="Appearance">
              <SettingsRow label="Theme">
                <ThemeSwitcher />
              </SettingsRow>
              {frame.translucent && (
                <SettingsRow label="Translucent" labelled>
                  <TranslucentSwitch />
                </SettingsRow>
              )}
            </SettingsSection>
            <StatsSettings />
            <UsageSettings />
            <ShellSettings />
            <NotificationSettings />
            <AccessibilitySettings />
            <MachineSettings
              onAdd={() => {
                setOpen(false);
              }}
            />
          </div>
        </ScrollArea>
      </DialogContent>
    </Dialog>
  );
}
