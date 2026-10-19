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
          <GearSixIcon weight={open ? "fill" : "regular"} />
        </TooltipTrigger>
        <TooltipContent side="top">Settings</TooltipContent>
      </Tooltip>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
        </DialogHeader>
        <section
          className="flex flex-col items-start gap-4"
          aria-labelledby="appearance-heading"
        >
          <h2 id="appearance-heading" className="type-subtitle">
            Appearance
          </h2>
          <ThemeSwitcher />
        </section>
        <MachineSettings
          onAdd={() => {
            setOpen(false);
          }}
        />
      </DialogContent>
    </Dialog>
  );
}
