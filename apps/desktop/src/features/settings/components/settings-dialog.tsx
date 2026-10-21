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
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { MachineSettings } from "@/features/settings/components/machine-settings";
import { NotificationSettings } from "@/features/settings/components/notification-settings";
import {
  SettingsRow,
  SettingsSection,
} from "@/features/settings/components/settings-section";

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
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
        </DialogHeader>
        <SettingsSection id="appearance-heading" title="Appearance">
          <SettingsRow label="Theme">
            <ThemeSwitcher />
          </SettingsRow>
        </SettingsSection>
        <NotificationSettings />
        <MachineSettings
          onAdd={() => {
            setOpen(false);
          }}
        />
      </DialogContent>
    </Dialog>
  );
}
