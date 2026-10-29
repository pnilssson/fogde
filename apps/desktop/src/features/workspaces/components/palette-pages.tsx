import { GithubLogoIcon, NotebookIcon, PulseIcon } from "@phosphor-icons/react";

import { CommandGroup, CommandItem } from "@/components/ui/command";
import { openPage } from "@/lib/page";

interface Props {
  machine: string | undefined;
  run: (action: () => void) => void;
}

export function PalettePages({ machine, run }: Props) {
  return (
    <CommandGroup heading="Pages">
      <CommandItem
        value="GitHub"
        keywords={[
          "open",
          "view",
          "show",
          "pull",
          "requests",
          "issues",
          "commit",
          "push",
        ]}
        onSelect={() => {
          run(() => {
            openPage("github", machine);
          });
        }}
      >
        <GithubLogoIcon />
        GitHub
      </CommandItem>
      <CommandItem
        value="Notes"
        keywords={["open", "view", "show"]}
        onSelect={() => {
          run(() => {
            openPage("notes", machine);
          });
        }}
      >
        <NotebookIcon />
        Notes
      </CommandItem>
      <CommandItem
        value="Monitor"
        keywords={["open", "view", "show", "cpu", "memory", "usage"]}
        onSelect={() => {
          run(() => {
            openPage("monitor", machine);
          });
        }}
      >
        <PulseIcon />
        Monitor
      </CommandItem>
    </CommandGroup>
  );
}
