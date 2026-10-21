import { MagnifyingGlassIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import { Keys } from "@/components/keys";
import { setPaletteOpen } from "@/features/workspaces/lib/palette";
import { shortcuts } from "@/lib/shortcuts";

export function SearchButton({ onOpen }: { onOpen: () => void }) {
  return (
    <Button
      variant="secondary"
      className="w-full justify-start text-muted-foreground"
      onClick={() => {
        onOpen();
        setPaletteOpen(true);
      }}
    >
      <MagnifyingGlassIcon data-icon="inline-start" />
      Search
      <Keys letter={shortcuts.palette} className="ml-auto" />
    </Button>
  );
}
