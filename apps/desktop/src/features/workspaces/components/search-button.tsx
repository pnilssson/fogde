import { MagnifyingGlassIcon } from "@phosphor-icons/react";

import { Keys } from "@/components/keys";
import { Button } from "@/components/ui/button";
import { setPaletteOpen } from "@/features/workspaces/lib/palette";
import { shortcuts } from "@/lib/shortcuts";

export function SearchButton() {
  return (
    <Button
      variant="ghost"
      size="sm"
      aria-label="Search"
      className="w-72 justify-start bg-surface-2 text-muted-foreground shadow-surface-1 hover:bg-hover max-md:w-auto"
      onClick={() => {
        setPaletteOpen(true);
      }}
    >
      <MagnifyingGlassIcon />
      <span className="flex-1 text-left type-body max-md:hidden">Search</span>
      <Keys
        letter={shortcuts.palette}
        className="bg-transparent px-0 max-md:hidden"
      />
    </Button>
  );
}
