import { DotsThreeIcon } from "@phosphor-icons/react";
import { Fragment } from "react";

import { Keys } from "@/components/keys";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
} from "@/components/ui/context-menu";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SidebarMenuAction } from "@/components/ui/sidebar";

export interface RowAction {
  label: string;
  onSelect: () => void;
  disabled?: boolean;
  destructive?: boolean;
  shortcut?: string;
}

type Groups = RowAction[][];

function shown(groups: Groups) {
  return groups.filter((group) => group.length > 0);
}

export function RowContext({ groups }: { groups: Groups }) {
  return (
    <ContextMenuContent>
      {shown(groups).map((group, index) => (
        <Fragment key={group[0]?.label}>
          {index > 0 && <ContextMenuSeparator />}
          {group.map((action) => (
            <ContextMenuItem
              key={action.label}
              disabled={action.disabled ?? false}
              variant={action.destructive ? "destructive" : "default"}
              onClick={action.onSelect}
            >
              {action.label}
              {action.shortcut && (
                <ContextMenuShortcut>
                  <Keys letter={action.shortcut} />
                </ContextMenuShortcut>
              )}
            </ContextMenuItem>
          ))}
        </Fragment>
      ))}
    </ContextMenuContent>
  );
}

export function RowActions({ name, groups }: { name: string; groups: Groups }) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <SidebarMenuAction
            aria-label={`Actions for ${name}`}
            className="pointer-events-none opacity-0 focus-visible:pointer-events-auto focus-visible:opacity-100 aria-expanded:opacity-100"
          />
        }
      >
        <DotsThreeIcon />
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        {shown(groups).map((group, index) => (
          <Fragment key={group[0]?.label}>
            {index > 0 && <DropdownMenuSeparator />}
            {group.map((action) => (
              <DropdownMenuItem
                key={action.label}
                disabled={action.disabled ?? false}
                variant={action.destructive ? "destructive" : "default"}
                onClick={action.onSelect}
              >
                {action.label}
                {action.shortcut && (
                  <DropdownMenuShortcut>
                    <Keys letter={action.shortcut} />
                  </DropdownMenuShortcut>
                )}
              </DropdownMenuItem>
            ))}
          </Fragment>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
