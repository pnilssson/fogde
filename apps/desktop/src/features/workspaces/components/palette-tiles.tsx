import {
  ArrowDownIcon,
  ArrowLeftIcon,
  ArrowRightIcon,
  ArrowUpIcon,
  SquareIcon,
  XIcon,
} from "@phosphor-icons/react";

import { Keys } from "@/components/keys";
import {
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from "@/components/ui/command";
import { closeActive } from "@/features/workspaces/lib/closing";
import { moveTile, type Side } from "@/features/workspaces/lib/drag";
import { focusTile } from "@/features/workspaces/lib/tiles";
import { arrows, shortcuts } from "@/lib/shortcuts";

const moves: { side: Side; label: string; icon: typeof ArrowLeftIcon }[] = [
  { side: "left", label: "Move tile left", icon: ArrowLeftIcon },
  { side: "right", label: "Move tile right", icon: ArrowRightIcon },
  { side: "top", label: "Move tile up", icon: ArrowUpIcon },
  { side: "bottom", label: "Move tile down", icon: ArrowDownIcon },
];

function glyph(side: Side) {
  return arrows.find((arrow) => arrow.side === side)?.glyph ?? "";
}

interface Props {
  tiles: number;
  run: (action: () => void) => void;
}

export function PaletteTiles({ tiles, run }: Props) {
  return (
    <CommandGroup heading="Tile">
      <CommandItem
        value="Close tile"
        onSelect={() => {
          run(closeActive);
        }}
      >
        <XIcon />
        Close tile
        <CommandShortcut>
          <Keys letter={shortcuts.close} />
        </CommandShortcut>
      </CommandItem>
      {tiles > 1 &&
        moves.map(({ side, label, icon: Icon }) => (
          <CommandItem
            key={side}
            value={label}
            onSelect={() => {
              run(() => {
                moveTile(side);
              });
            }}
          >
            <Icon />
            {label}
            <CommandShortcut>
              <Keys letter={glyph(side)} alt />
            </CommandShortcut>
          </CommandItem>
        ))}
      {tiles > 1 &&
        Array.from({ length: Math.min(tiles, 9) }, (_, index) => {
          const number = index + 1;

          return (
            <CommandItem
              key={number}
              value={`Go to tile ${String(number)}`}
              onSelect={() => {
                run(() => {
                  focusTile(number);
                });
              }}
            >
              <SquareIcon />
              Go to tile {number}
              <CommandShortcut>
                <Keys letter={String(number)} />
              </CommandShortcut>
            </CommandItem>
          );
        })}
    </CommandGroup>
  );
}
