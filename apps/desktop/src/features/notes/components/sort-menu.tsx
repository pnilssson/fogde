import { SortAscendingIcon } from "@phosphor-icons/react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { setOrder, useOrder } from "@/features/notes/lib/sort";
import { type Order, orders } from "@/features/notes/lib/tree";

const labels: Record<Order, string> = {
  "name-asc": "File name (A to Z)",
  "name-desc": "File name (Z to A)",
  "modified-desc": "Modified time (new to old)",
  "modified-asc": "Modified time (old to new)",
  "created-desc": "Created time (new to old)",
  "created-asc": "Created time (old to new)",
};

export function SortMenu() {
  const order = useOrder();

  return (
    <DropdownMenu>
      <Tooltip>
        <TooltipTrigger
          render={
            <DropdownMenuTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label="Change sort order"
                />
              }
            />
          }
        >
          <SortAscendingIcon />
        </TooltipTrigger>
        <TooltipContent>Change sort order</TooltipContent>
      </Tooltip>
      <DropdownMenuContent align="start" className="w-auto">
        <DropdownMenuRadioGroup
          value={order}
          onValueChange={(value: Order) => {
            setOrder(value);
          }}
        >
          {orders.map((option, index) => (
            <Option
              key={option}
              option={option}
              split={index % 2 === 0 && index > 0}
            />
          ))}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function Option({ option, split }: { option: Order; split: boolean }) {
  return (
    <>
      {split && <DropdownMenuSeparator />}
      <DropdownMenuRadioItem value={option}>
        {labels[option]}
      </DropdownMenuRadioItem>
    </>
  );
}
